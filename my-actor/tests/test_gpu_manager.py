import pytest
import torch
import asyncio
from unittest.mock import patch, MagicMock
from src.main import GPUManager

@pytest.fixture
def gpu_manager():
    with patch('torch.cuda.device_count', return_value=2):
        yield GPUManager()

def test_init_with_gpus(gpu_manager):
    assert gpu_manager.available_gpus == 2
    assert gpu_manager.current_gpu == 0

def test_init_without_gpus():
    with patch('torch.cuda.device_count', return_value=0):
        with pytest.raises(RuntimeError, match="No GPU devices available"):
            GPUManager()

def test_get_next_gpu(gpu_manager):
    # First call should return 0
    assert gpu_manager.get_next_gpu() == 0
    # Second call should return 1
    assert gpu_manager.get_next_gpu() == 1
    # Third call should wrap around to 0
    assert gpu_manager.get_next_gpu() == 0

def test_get_next_gpu_concurrent(gpu_manager):
    """Test concurrent access to get_next_gpu."""
    results = []
    for _ in range(10):
        results.append(gpu_manager.get_next_gpu())

    # Should alternate between 0 and 1
    expected = [0, 1] * 5
    assert results == expected

def test_get_next_gpu_after_failure():
    """Test GPU selection behavior after a GPU failure."""
    with patch('torch.cuda.device_count', return_value=2) as mock_count:
        manager = GPUManager()

        # Simulate first GPU failure
        with patch('torch.cuda.device', side_effect=torch.cuda.CudaError):
            # Should skip to next GPU
            assert manager.get_next_gpu() == 1

        # Should continue from last successful GPU
        assert manager.get_next_gpu() == 0

@pytest.mark.asyncio
async def test_update_metrics(gpu_manager):
    mock_gpu = MagicMock()
    mock_gpu.id = 0
    mock_gpu.load = 0.75
    mock_gpu.memoryUsed = 1000

    with patch('GPUtil.getGPUs', return_value=[mock_gpu]):
        # Create a task for update_metrics
        task = asyncio.create_task(gpu_manager.update_metrics())

        # Wait a short time for metrics to update
        await asyncio.sleep(0.1)

        # Cancel the task
        task.cancel()
        try:
            await task
        except asyncio.CancelledError:
            pass

        # Verify metrics were updated
        from prometheus_client import REGISTRY
        gpu_util = REGISTRY.get_sample_value('gpu_utilization_percent', {'gpu_id': '0'})
        gpu_mem = REGISTRY.get_sample_value('gpu_memory_used_mb', {'gpu_id': '0'})

        assert gpu_util == 75.0  # 0.75 * 100
        assert gpu_mem == 1000.0

@pytest.mark.asyncio
async def test_update_metrics_multiple_gpus():
    """Test metrics update with multiple GPUs."""
    mock_gpu1 = MagicMock()
    mock_gpu1.id = 0
    mock_gpu1.load = 0.75
    mock_gpu1.memoryUsed = 1000

    mock_gpu2 = MagicMock()
    mock_gpu2.id = 1
    mock_gpu2.load = 0.50
    mock_gpu2.memoryUsed = 500

    with patch('torch.cuda.device_count', return_value=2):
        manager = GPUManager()

        with patch('GPUtil.getGPUs', return_value=[mock_gpu1, mock_gpu2]):
            task = asyncio.create_task(manager.update_metrics())
            await asyncio.sleep(0.1)
            task.cancel()

            try:
                await task
            except asyncio.CancelledError:
                pass

            from prometheus_client import REGISTRY

            # Check first GPU metrics
            gpu1_util = REGISTRY.get_sample_value('gpu_utilization_percent', {'gpu_id': '0'})
            gpu1_mem = REGISTRY.get_sample_value('gpu_memory_used_mb', {'gpu_id': '0'})
            assert gpu1_util == 75.0
            assert gpu1_mem == 1000.0

            # Check second GPU metrics
            gpu2_util = REGISTRY.get_sample_value('gpu_utilization_percent', {'gpu_id': '1'})
            gpu2_mem = REGISTRY.get_sample_value('gpu_memory_used_mb', {'gpu_id': '1'})
            assert gpu2_util == 50.0
            assert gpu2_mem == 500.0

@pytest.mark.asyncio
async def test_update_metrics_gpu_error():
    """Test metrics update when GPU information cannot be retrieved."""
    def mock_gpu_error(*args, **kwargs):
        raise Exception("Failed to get GPU info")

    with patch('torch.cuda.device_count', return_value=2):
        manager = GPUManager()

        with patch('GPUtil.getGPUs', side_effect=mock_gpu_error):
            task = asyncio.create_task(manager.update_metrics())
            await asyncio.sleep(0.1)
            task.cancel()

            try:
                await task
            except asyncio.CancelledError:
                pass

            # Metrics should not be updated
            from prometheus_client import REGISTRY
            gpu_util = REGISTRY.get_sample_value('gpu_utilization_percent', {'gpu_id': '0'})
            assert gpu_util is None
