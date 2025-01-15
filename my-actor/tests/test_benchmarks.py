import pytest
import asyncio
import time
import torch
import numpy as np
from PIL import Image
from src.main import Actor, GPUManager, ModelManager, ProcessingQueue

@pytest.fixture
def large_batch_size():
    return 32

@pytest.fixture
def sample_texts():
    """Generate a large batch of sample texts."""
    return [
        f"This is a sample text {i} for sentiment analysis and NER testing."
        for i in range(1000)
    ]

@pytest.fixture
def sample_images():
    """Generate a large batch of sample images."""
    return [
        Image.fromarray(np.random.randint(0, 255, (224, 224, 3), dtype=np.uint8))
        for _ in range(100)
    ]

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_model_loading_performance(benchmark):
    """Benchmark model loading time."""
    with patch('torch.cuda.device_count', return_value=2):
        gpu_manager = GPUManager()
        model_manager = ModelManager(gpu_manager)

        def load_models():
            asyncio.run(model_manager.get_model('bert-base-uncased', 'sentiment'))
            asyncio.run(model_manager.get_model('bert-base-ner', 'ner'))

        # Run benchmark
        result = benchmark(load_models)

        # Verify models were loaded successfully
        assert len(model_manager.models) == 2

        # Cleanup
        model_manager.cleanup()

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_batch_processing_performance(benchmark, sample_texts, large_batch_size):
    """Benchmark batch processing performance."""
    with patch('torch.cuda.device_count', return_value=2):
        gpu_manager = GPUManager()
        model_manager = ModelManager(gpu_manager)
        queue = ProcessingQueue(batch_size=large_batch_size)

        # Load models
        sentiment_model = await model_manager.get_model('bert-base-uncased', 'sentiment')

        def process_batch():
            # Add texts to queue
            for text in sample_texts[:large_batch_size]:
                queue.add_text(text)

            # Process batch
            batch = queue.get_text_batch()
            return asyncio.run(model_manager.process_batch(sentiment_model, batch, 'sentiment'))

        # Run benchmark
        result = benchmark(process_batch)

        # Verify processing completed successfully
        assert len(result) == large_batch_size

        # Cleanup
        model_manager.cleanup()

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_multi_gpu_scaling(benchmark, sample_texts):
    """Benchmark scaling across multiple GPUs."""
    batch_sizes = [8, 16, 32, 64]
    gpu_counts = [1, 2, 4]  # Test with different GPU counts
    results = {}

    for gpu_count in gpu_counts:
        with patch('torch.cuda.device_count', return_value=gpu_count):
            gpu_manager = GPUManager()
            model_manager = ModelManager(gpu_manager)

            # Load model
            model = await model_manager.get_model('bert-base-uncased', 'sentiment', use_parallel=True)

            for batch_size in batch_sizes:
                queue = ProcessingQueue(batch_size=batch_size)

                def process_parallel_batch():
                    # Add texts to queue
                    for text in sample_texts[:batch_size]:
                        queue.add_text(text)

                    # Process batch
                    batch = queue.get_text_batch()
                    return asyncio.run(model_manager.process_batch(model, batch, 'sentiment'))

                # Run benchmark
                result = benchmark(process_parallel_batch)
                results[(gpu_count, batch_size)] = result.stats.mean

            # Cleanup
            model_manager.cleanup()

    # Verify scaling efficiency
    for batch_size in batch_sizes:
        single_gpu_time = results.get((1, batch_size))
        if single_gpu_time:
            for gpu_count in gpu_counts[1:]:
                multi_gpu_time = results.get((gpu_count, batch_size))
                if multi_gpu_time:
                    speedup = single_gpu_time / multi_gpu_time
                    # Verify reasonable scaling (at least 50% efficiency)
                    assert speedup >= 0.5 * gpu_count

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_memory_usage(benchmark, sample_texts, large_batch_size):
    """Benchmark memory usage during processing."""
    with patch('torch.cuda.device_count', return_value=2):
        gpu_manager = GPUManager()
        model_manager = ModelManager(gpu_manager)
        queue = ProcessingQueue(batch_size=large_batch_size)

        # Load model
        model = await model_manager.get_model('bert-base-uncased', 'sentiment')

        def measure_memory_usage():
            initial_memory = torch.cuda.memory_allocated()

            # Process batch
            for text in sample_texts[:large_batch_size]:
                queue.add_text(text)
            batch = queue.get_text_batch()
            asyncio.run(model_manager.process_batch(model, batch, 'sentiment'))

            peak_memory = torch.cuda.max_memory_allocated()
            torch.cuda.reset_peak_memory_stats()

            return peak_memory - initial_memory

        # Run benchmark
        result = benchmark(measure_memory_usage)

        # Verify memory usage is within reasonable limits (e.g., < 8GB)
        assert result.stats.mean < 8 * 1024 * 1024 * 1024

        # Cleanup
        model_manager.cleanup()

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_end_to_end_latency(benchmark, mock_html_content, mock_httpx_client):
    """Benchmark end-to-end processing latency."""
    with patch('torch.cuda.device_count', return_value=2), \
         patch('httpx.AsyncClient', return_value=mock_httpx_client), \
         patch('prometheus_client.start_http_server'):

        def process_webpage():
            async with Actor:
                # Set up test input
                asyncio.run(Actor.set_input({'url': 'https://test.com'}))

                # Process webpage
                start_time = time.time()
                asyncio.run(main())
                end_time = time.time()

                return end_time - start_time

        # Run benchmark
        result = benchmark(process_webpage)

        # Verify latency is within acceptable range (e.g., < 5 seconds)
        assert result.stats.mean < 5.0

@pytest.mark.benchmark
@pytest.mark.asyncio
async def test_concurrent_processing(benchmark, sample_texts, large_batch_size):
    """Benchmark concurrent processing performance."""
    num_concurrent = 4

    with patch('torch.cuda.device_count', return_value=2):
        gpu_manager = GPUManager()
        model_manager = ModelManager(gpu_manager)
        queues = [ProcessingQueue(batch_size=large_batch_size) for _ in range(num_concurrent)]

        # Load model
        model = await model_manager.get_model('bert-base-uncased', 'sentiment')

        async def process_concurrent():
            tasks = []

            # Create processing tasks
            for queue in queues:
                for text in sample_texts[:large_batch_size]:
                    queue.add_text(text)
                batch = queue.get_text_batch()
                task = model_manager.process_batch(model, batch, 'sentiment')
                tasks.append(task)

            # Run tasks concurrently
            return await asyncio.gather(*tasks)

        def run_concurrent():
            return asyncio.run(process_concurrent())

        # Run benchmark
        result = benchmark(run_concurrent)

        # Verify all tasks completed successfully
        assert len(result.stats.iterations) == num_concurrent

        # Cleanup
        model_manager.cleanup()

if __name__ == '__main__':
    pytest.main(['--benchmark-only'])
