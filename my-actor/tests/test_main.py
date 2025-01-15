import pytest
import asyncio
import torch
import yaml
import json
from unittest.mock import patch, MagicMock, AsyncMock
from bs4 import BeautifulSoup
from PIL import Image
import io
import numpy as np
from src.main import Actor, GPUManager, ModelManager, ProcessingQueue

@pytest.fixture
def config():
    return {
        'gpu': {
            'use_mixed_precision': True,
            'batch_size': 4,
            'model_parallel_threshold': 1024
        },
        'models': {
            'sentiment': {
                'name': 'test-sentiment-model',
                'max_length': 512
            },
            'ner': {
                'name': 'test-ner-model',
                'max_length': 512
            }
        },
        'processing': {
            'max_concurrent_requests': 5,
            'request_timeout': 30
        }
    }

@pytest.fixture
def mock_html_content():
    return """
    <html>
        <body>
            <h1>Test Page</h1>
            <p>This is a test paragraph with some text for sentiment analysis.</p>
            <p>Another paragraph mentioning entities like Google and Microsoft.</p>
            <img src="test1.jpg" alt="Test Image 1">
            <img src="test2.jpg" alt="Test Image 2">
        </body>
    </html>
    """

@pytest.fixture
def mock_image():
    return Image.fromarray(np.zeros((64, 64, 3), dtype=np.uint8))

@pytest.fixture
def mock_httpx_client():
    async def mock_get(url):
        response = MagicMock()
        if url.endswith('.jpg'):
            response.content = io.BytesIO(np.zeros((64, 64, 3), dtype=np.uint8).tobytes())
        else:
            response.text = mock_html_content()
        response.raise_for_status = lambda: None
        return response

    client = AsyncMock()
    client.get = mock_get
    return client

@pytest.mark.asyncio
async def test_successful_processing(config, mock_html_content, mock_httpx_client):
    """Test successful end-to-end processing of a webpage."""

    # Mock GPU setup
    with patch('torch.cuda.device_count', return_value=2), \
         patch('torch.distributed.init_process_group'), \
         patch('httpx.AsyncClient', return_value=mock_httpx_client), \
         patch('prometheus_client.start_http_server'):

        # Mock model responses
        mock_sentiment = [{'label': 'POSITIVE', 'score': 0.9}]
        mock_ner = [{'entity': 'ORG', 'word': 'Google', 'score': 0.95}]

        with patch('transformers.pipeline', return_value=lambda x: mock_sentiment if 'sentiment' in str(x) else mock_ner):
            # Initialize actor with test config
            async with Actor:
                # Set up test input
                await Actor.set_input({'url': 'https://test.com'})

                # Create and run main processing task
                task = asyncio.create_task(main())

                # Wait for processing to complete
                await asyncio.sleep(0.1)
                task.cancel()

                try:
                    await task
                except asyncio.CancelledError:
                    pass

                # Verify results were pushed
                pushed_data = Actor.get_pushed_data()
                assert len(pushed_data) > 0

                # Verify text processing results
                text_results = [d for d in pushed_data if 'text' in d]
                assert len(text_results) > 0
                assert all('sentiment' in r for r in text_results)
                assert all('entities' in r for r in text_results)

                # Verify image processing results
                image_results = [d for d in pushed_data if 'image_url' in d]
                assert len(image_results) > 0
                assert all('analysis' in r for r in image_results)

@pytest.mark.asyncio
async def test_gpu_failure_handling(config, mock_html_content):
    """Test handling of GPU failures during processing."""

    def mock_cuda_error(*args, **kwargs):
        raise torch.cuda.CudaError("Simulated CUDA error")

    with patch('torch.cuda.device_count', return_value=2), \
         patch('torch.cuda.device', side_effect=mock_cuda_error), \
         patch('prometheus_client.start_http_server'):

        async with Actor:
            await Actor.set_input({'url': 'https://test.com'})

            with pytest.raises(torch.cuda.CudaError):
                await main()

@pytest.mark.asyncio
async def test_invalid_input_handling():
    """Test handling of invalid input."""

    with patch('torch.cuda.device_count', return_value=2), \
         patch('prometheus_client.start_http_server'):

        async with Actor:
            # Test with missing URL
            await Actor.set_input({})

            # Should use default URL
            task = asyncio.create_task(main())
            await asyncio.sleep(0.1)
            task.cancel()

            try:
                await task
            except asyncio.CancelledError:
                pass

            # Test with invalid URL format
            await Actor.set_input({'url': 'not-a-url'})

            with pytest.raises(Exception):
                await main()

@pytest.mark.asyncio
async def test_queue_overflow_handling(config, mock_html_content):
    """Test handling of processing queue overflow."""

    # Create a very large HTML document
    large_html = "<html><body>" + "<p>Test paragraph</p>" * 1000 + "</body></html>"

    with patch('torch.cuda.device_count', return_value=2), \
         patch('prometheus_client.start_http_server'):

        async with Actor:
            await Actor.set_input({'url': 'https://test.com'})

            # Initialize processing queue with small batch size
            queue = ProcessingQueue(batch_size=2)

            # Add many items to queue
            for _ in range(100):
                queue.add_text("test text")

            # Verify queue handles overflow gracefully
            batches = []
            while True:
                batch = queue.get_text_batch()
                if not batch:
                    break
                batches.append(batch)

            assert len(batches) == 50  # 100 items / batch size 2
            assert all(len(batch) == 2 for batch in batches)

@pytest.mark.asyncio
async def test_model_loading_failure(config):
    """Test handling of model loading failures."""

    def mock_model_error(*args, **kwargs):
        raise Exception("Failed to load model")

    with patch('torch.cuda.device_count', return_value=2), \
         patch('transformers.AutoModelForSequenceClassification.from_pretrained',
               side_effect=mock_model_error), \
         patch('prometheus_client.start_http_server'):

        async with Actor:
            await Actor.set_input({'url': 'https://test.com'})

            with pytest.raises(Exception, match="Failed to load model"):
                await main()

@pytest.mark.asyncio
async def test_metrics_recording(config, mock_html_content, mock_httpx_client):
    """Test that metrics are properly recorded during processing."""

    with patch('torch.cuda.device_count', return_value=2), \
         patch('httpx.AsyncClient', return_value=mock_httpx_client), \
         patch('prometheus_client.start_http_server'):

        async with Actor:
            await Actor.set_input({'url': 'https://test.com'})

            # Run processing
            task = asyncio.create_task(main())
            await asyncio.sleep(0.1)
            task.cancel()

            try:
                await task
            except asyncio.CancelledError:
                pass

            # Verify metrics were recorded
            from prometheus_client import REGISTRY

            # Check text processing metrics
            texts_processed = REGISTRY.get_sample_value('processed_texts_total')
            assert texts_processed > 0

            # Check image processing metrics
            images_processed = REGISTRY.get_sample_value('processed_images_total')
            assert images_processed > 0

            # Check GPU metrics
            gpu_util = REGISTRY.get_sample_value('gpu_utilization_percent', {'gpu_id': '0'})
            assert gpu_util is not None

            gpu_mem = REGISTRY.get_sample_value('gpu_memory_used_mb', {'gpu_id': '0'})
            assert gpu_mem is not None
