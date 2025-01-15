"""
Enhanced Apify Actor with GPU acceleration and advanced multimedia processing capabilities.
"""

import os
import time
import logging
import asyncio
import json
import io
from pathlib import Path
from typing import Dict, List, Optional

import torch
import torch.cuda.amp as amp
import numpy as np
from bs4 import BeautifulSoup
import httpx
from apify import Actor
from prometheus_client import start_http_server, Counter, Gauge
import GPUtil
from PIL import Image
import cv2
import torchvision.transforms.functional as F
from transformers import (
    AutoTokenizer,
    AutoModelForSequenceClassification,
    AutoModelForTokenClassification,
    pipeline
)

# Initialize logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Prometheus metrics
METRICS_PORT = 9091
processed_texts = Counter('processed_texts_total', 'Number of processed text items')
processed_images = Counter('processed_images_total', 'Number of processed images')
gpu_utilization = Gauge('gpu_utilization_percent', 'GPU utilization percentage', ['gpu_id'])
gpu_memory = Gauge('gpu_memory_used_mb', 'GPU memory used in MB', ['gpu_id'])
processing_time = Gauge('processing_time_seconds', 'Processing time in seconds', ['task_type'])

class GPUManager:
    """Manages GPU resources and monitors utilization."""

    def __init__(self):
        self.available_gpus = torch.cuda.device_count()
        if self.available_gpus == 0:
            raise RuntimeError("No GPU devices available")

        logger.info(f"Found {self.available_gpus} GPU devices")
        self.current_gpu = 0

    def get_next_gpu(self) -> int:
        """Returns the next available GPU in round-robin fashion."""
        gpu = self.current_gpu
        self.current_gpu = (self.current_gpu + 1) % self.available_gpus
        return gpu

    async def update_metrics(self):
        """Updates Prometheus metrics for GPU utilization."""
        while True:
            for gpu in GPUtil.getGPUs():
                gpu_utilization.labels(gpu_id=gpu.id).set(gpu.load * 100)
                gpu_memory.labels(gpu_id=gpu.id).set(gpu.memoryUsed)
            await asyncio.sleep(1)

class ModelManager:
    """Manages ML models and ensures efficient GPU memory usage."""

    def __init__(self, gpu_manager: GPUManager):
        self.gpu_manager = gpu_manager
        self.models = {}
        self.scaler = amp.GradScaler()

        # Initialize distributed training if multiple GPUs are available
        self.world_size = torch.cuda.device_count()
        if self.world_size > 1:
            torch.distributed.init_process_group(
                backend='nccl',
                init_method='env://',
                world_size=self.world_size,
                rank=0
            )
            logger.info(f"Initialized distributed training with {self.world_size} GPUs")

    async def get_model(self, model_name: str, task_type: str, use_parallel: bool = True) -> torch.nn.Module:
        """Loads and returns a model, managing GPU placement."""
        if model_name not in self.models:
            gpu_id = self.gpu_manager.get_next_gpu()
            with torch.cuda.device(gpu_id):
                if task_type == 'sentiment':
                    model = AutoModelForSequenceClassification.from_pretrained(model_name)
                elif task_type == 'ner':
                    model = AutoModelForTokenClassification.from_pretrained(model_name)
                else:
                    raise ValueError(f"Unknown task type: {task_type}")

                # Move model to GPU
                model.cuda()

                # Apply model parallelism if model is large and multiple GPUs are available
                if use_parallel and self.world_size > 1:
                    if model.config.hidden_size >= 1024:  # Large model threshold
                        logger.info(f"Using model parallelism for {model_name}")
                        model = torch.nn.parallel.DistributedDataParallel(
                            model,
                            device_ids=[gpu_id],
                            output_device=gpu_id,
                            find_unused_parameters=True
                        )
                    else:
                        logger.info(f"Using data parallelism for {model_name}")
                        model = torch.nn.DataParallel(model)

                self.models[model_name] = {
                    'model': model,
                    'gpu_id': gpu_id
                }

        return self.models[model_name]['model']

    def cleanup(self):
        """Cleanup distributed training resources."""
        if self.world_size > 1:
            torch.distributed.destroy_process_group()

    async def process_batch(self, model: torch.nn.Module, batch: List[str], task_type: str) -> List[Dict]:
        """Process a batch of data using the model with automatic mixed precision."""
        with torch.cuda.amp.autocast():
            if isinstance(model, (torch.nn.DataParallel, torch.nn.parallel.DistributedDataParallel)):
                # Split batch across GPUs
                batch_size = len(batch)
                chunks = torch.chunk(torch.tensor(range(batch_size)), self.world_size)
                results = []

                for i, chunk in enumerate(chunks):
                    with torch.cuda.device(i):
                        chunk_batch = [batch[j] for j in chunk]
                        if task_type == 'sentiment':
                            chunk_results = pipeline('sentiment-analysis', model=model)(chunk_batch)
                        else:  # NER
                            chunk_results = pipeline('ner', model=model)(chunk_batch)
                        results.extend(chunk_results)

                return results
            else:
                # Single GPU processing
                if task_type == 'sentiment':
                    return pipeline('sentiment-analysis', model=model)(batch)
                else:  # NER
                    return pipeline('ner', model=model)(batch)

class ProcessingQueue:
    """Manages batched processing of items for efficient GPU utilization."""

    def __init__(self, batch_size: int = 8):
        self.batch_size = batch_size
        self.text_queue = []
        self.image_queue = []

    def add_text(self, text: str):
        """Adds text to the processing queue."""
        self.text_queue.append(text)

    def add_image(self, image: Image.Image):
        """Adds image to the processing queue."""
        self.image_queue.append(image)

    def get_text_batch(self) -> List[str]:
        """Returns a batch of texts for processing."""
        if len(self.text_queue) >= self.batch_size:
            batch = self.text_queue[:self.batch_size]
            self.text_queue = self.text_queue[self.batch_size:]
            return batch
        return []

    def get_image_batch(self) -> List[Image.Image]:
        """Returns a batch of images for processing."""
        if len(self.image_queue) >= self.batch_size:
            batch = self.image_queue[:self.batch_size]
            self.image_queue = self.image_queue[self.batch_size:]
            return batch
        return []

async def main():
    """Main entry point for the Apify Actor."""
    async with Actor:
        # Start Prometheus metrics server
        start_http_server(METRICS_PORT)

        # Initialize managers
        gpu_manager = GPUManager()
        model_manager = ModelManager(gpu_manager)
        queue = ProcessingQueue()

        # Start GPU metrics update task
        asyncio.create_task(gpu_manager.update_metrics())

        # Get input
        actor_input = await Actor.get_input() or {}
        url = actor_input.get('url', 'https://example.com')

        logger.info(f"Processing URL: {url}")
        start_time = time.time()

        try:
            # Fetch webpage content
            async with httpx.AsyncClient() as client:
                response = await client.get(url)
                response.raise_for_status()
                html_content = response.text

            # Parse HTML
            soup = BeautifulSoup(html_content, 'lxml')

            # Process text content
            sentiment_model = await model_manager.get_model('distilbert-base-uncased-finetuned-sst-2-english', 'sentiment')
            ner_model = await model_manager.get_model('dbmdz/bert-large-cased-finetuned-conll03-english', 'ner')

            # Extract and process text
            for paragraph in soup.find_all(['p', 'h1', 'h2', 'h3']):
                text = paragraph.get_text().strip()
                if text:
                    queue.add_text(text)

                    # Process batch if available
                    if batch := queue.get_text_batch():
                        with torch.cuda.amp.autocast():
                            # Perform sentiment analysis
                            sentiments = pipeline('sentiment-analysis', model=sentiment_model)(batch)

                            # Perform named entity recognition
                            entities = pipeline('ner', model=ner_model)(batch)

                        # Save results
                        for text, sentiment, text_entities in zip(batch, sentiments, entities):
                            await Actor.push_data({
                                'text': text,
                                'sentiment': sentiment,
                                'entities': text_entities
                            })
                            processed_texts.inc()

            # Process images
            for img_tag in soup.find_all('img'):
                img_url = img_tag.get('src')
                if img_url:
                    try:
                        async with httpx.AsyncClient() as client:
                            response = await client.get(img_url)
                            img_data = response.content
                            image = Image.open(io.BytesIO(img_data))
                            queue.add_image(image)

                            # Process batch if available
                            if batch := queue.get_image_batch():
                                with torch.cuda.amp.autocast():
                                    # Convert images to tensors
                                    tensors = [F.to_tensor(img).unsqueeze(0) for img in batch]
                                    batch_tensor = torch.cat(tensors).cuda()

                                    # Perform image analysis (example with basic stats)
                                    results = {
                                        'mean': batch_tensor.mean((2, 3)).tolist(),
                                        'std': batch_tensor.std((2, 3)).tolist(),
                                        'max': batch_tensor.max((2, 3))[0].tolist()
                                    }

                                await Actor.push_data({
                                    'image_url': img_url,
                                    'analysis': results
                                })
                                processed_images.inc()

                    except Exception as e:
                        logger.error(f"Error processing image {img_url}: {str(e)}")

            # Record processing time
            total_time = time.time() - start_time
            processing_time.labels(task_type='total').set(total_time)

            logger.info(f"Processing completed in {total_time:.2f} seconds")

        except Exception as e:
            logger.error(f"Error processing URL {url}: {str(e)}")
            raise

if __name__ == "__main__":
    asyncio.run(main())
