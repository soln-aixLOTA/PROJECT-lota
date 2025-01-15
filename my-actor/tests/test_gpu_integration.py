import pytest
import torch
import asyncio
import numpy as np
from PIL import Image
from src.main import GPUManager, ModelManager, ProcessingQueue

def requires_gpu(func):
    """Decorator to skip tests if no GPU is available."""
    return pytest.mark.skipif(
        not torch.cuda.is_available(),
        reason="Test requires GPU"
    )(func)

@pytest.fixture
def real_gpu_manager():
    """Create a GPUManager using real GPUs."""
    if not torch.cuda.is_available():
        pytest.skip("No GPU available")
    return GPUManager()

@pytest.fixture
def real_model_manager(real_gpu_manager):
    """Create a ModelManager using real GPUs."""
    if not torch.cuda.is_available():
        pytest.skip("No GPU available")
    return ModelManager(real_gpu_manager)

@requires_gpu
@pytest.mark.asyncio
async def test_real_model_loading(real_model_manager):
    """Test loading models on real GPU."""
    # Load sentiment model
    sentiment_model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment'
    )
    assert sentiment_model is not None
    assert next(sentiment_model.parameters()).is_cuda

    # Load NER model
    ner_model = await real_model_manager.get_model(
        'dbmdz/bert-large-cased-finetuned-conll03-english',
        'ner'
    )
    assert ner_model is not None
    assert next(ner_model.parameters()).is_cuda

    # Cleanup
    real_model_manager.cleanup()

@requires_gpu
@pytest.mark.asyncio
async def test_real_batch_processing(real_model_manager):
    """Test processing batches on real GPU."""
    # Prepare test data
    texts = [
        "This is a very positive review!",
        "I'm feeling quite negative about this.",
        "The weather is nice today."
    ]

    # Load model
    model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment'
    )

    # Process batch
    results = await real_model_manager.process_batch(model, texts, 'sentiment')

    # Verify results
    assert len(results) == len(texts)
    assert all('label' in r and 'score' in r for r in results)

    # Cleanup
    real_model_manager.cleanup()

@requires_gpu
@pytest.mark.asyncio
async def test_real_multi_gpu_processing(real_model_manager):
    """Test processing on multiple GPUs if available."""
    if torch.cuda.device_count() < 2:
        pytest.skip("Test requires at least 2 GPUs")

    # Prepare large batch
    texts = [f"Sample text {i}" for i in range(100)]

    # Load model with parallel processing
    model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment',
        use_parallel=True
    )

    # Process batch
    results = await real_model_manager.process_batch(model, texts, 'sentiment')

    # Verify results
    assert len(results) == len(texts)

    # Cleanup
    real_model_manager.cleanup()

@requires_gpu
@pytest.mark.asyncio
async def test_real_gpu_memory_management(real_model_manager):
    """Test GPU memory management with real operations."""
    # Record initial memory
    initial_memory = torch.cuda.memory_allocated()

    # Load model
    model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment'
    )

    # Record memory after model load
    model_memory = torch.cuda.memory_allocated()
    assert model_memory > initial_memory

    # Process some data
    texts = ["Test text"] * 10
    results = await real_model_manager.process_batch(model, texts, 'sentiment')

    # Record peak memory
    peak_memory = torch.cuda.max_memory_allocated()

    # Cleanup
    real_model_manager.cleanup()

    # Verify memory was freed
    final_memory = torch.cuda.memory_allocated()
    assert final_memory < peak_memory

@requires_gpu
@pytest.mark.asyncio
async def test_real_gpu_error_recovery(real_model_manager):
    """Test recovery from GPU errors with real hardware."""
    # Force GPU out of memory error
    try:
        # Attempt to allocate more memory than available
        large_tensor = torch.zeros(int(1e10), device='cuda')
    except RuntimeError as e:
        # Verify we can still use GPU after error
        model = await real_model_manager.get_model(
            'distilbert-base-uncased-finetuned-sst-2-english',
            'sentiment'
        )
        results = await real_model_manager.process_batch(
            model,
            ["Test text"],
            'sentiment'
        )
        assert len(results) == 1

    # Cleanup
    real_model_manager.cleanup()

@requires_gpu
@pytest.mark.asyncio
async def test_real_mixed_precision(real_model_manager):
    """Test mixed precision training with real GPU."""
    # Load model
    model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment'
    )

    # Process with mixed precision
    with torch.cuda.amp.autocast():
        results = await real_model_manager.process_batch(
            model,
            ["Test text"],
            'sentiment'
        )

    assert len(results) == 1

    # Cleanup
    real_model_manager.cleanup()

@requires_gpu
@pytest.mark.asyncio
async def test_real_concurrent_gpu_ops(real_model_manager):
    """Test concurrent GPU operations."""
    # Load models
    sentiment_model = await real_model_manager.get_model(
        'distilbert-base-uncased-finetuned-sst-2-english',
        'sentiment'
    )
    ner_model = await real_model_manager.get_model(
        'dbmdz/bert-large-cased-finetuned-conll03-english',
        'ner'
    )

    # Create concurrent tasks
    texts = ["Test text"] * 5
    tasks = [
        real_model_manager.process_batch(sentiment_model, texts, 'sentiment'),
        real_model_manager.process_batch(ner_model, texts, 'ner')
    ]

    # Run concurrently
    results = await asyncio.gather(*tasks)

    # Verify results
    assert len(results) == 2
    assert len(results[0]) == len(texts)  # Sentiment results
    assert len(results[1]) == len(texts)  # NER results

    # Cleanup
    real_model_manager.cleanup()

if __name__ == '__main__':
    pytest.main(['-v', '--gpu'])
