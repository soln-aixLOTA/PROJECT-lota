import pytest
import torch
import asyncio
from unittest.mock import patch, MagicMock
from src.main import ModelManager, GPUManager

@pytest.fixture
def gpu_manager():
    with patch('torch.cuda.device_count', return_value=2):
        yield GPUManager()

@pytest.fixture
def model_manager(gpu_manager):
    with patch('torch.distributed.init_process_group'):
        yield ModelManager(gpu_manager)

@pytest.mark.asyncio
async def test_get_model_sentiment(model_manager):
    mock_model = MagicMock()
    mock_model.config.hidden_size = 768

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_model):
        model = await model_manager.get_model('test-model', 'sentiment')
        assert model is not None
        assert 'test-model' in model_manager.models

@pytest.mark.asyncio
async def test_get_model_ner(model_manager):
    mock_model = MagicMock()
    mock_model.config.hidden_size = 768

    with patch('transformers.AutoModelForTokenClassification.from_pretrained', return_value=mock_model):
        model = await model_manager.get_model('test-model', 'ner')
        assert model is not None
        assert 'test-model' in model_manager.models

@pytest.mark.asyncio
async def test_get_model_invalid_task(model_manager):
    with pytest.raises(ValueError, match="Unknown task type: invalid"):
        await model_manager.get_model('test-model', 'invalid')

@pytest.mark.asyncio
async def test_model_parallelism(model_manager):
    mock_model = MagicMock()
    mock_model.config.hidden_size = 1024  # Large model

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_model), \
         patch('torch.nn.parallel.DistributedDataParallel', return_value=mock_model):
        model = await model_manager.get_model('test-model', 'sentiment', use_parallel=True)
        assert model is not None

@pytest.mark.asyncio
async def test_process_batch(model_manager):
    mock_model = MagicMock()
    mock_results = [{'label': 'POSITIVE', 'score': 0.9}]

    with patch('transformers.pipeline', return_value=lambda x: mock_results):
        results = await model_manager.process_batch(mock_model, ['test text'], 'sentiment')
        assert results == mock_results

def test_cleanup(model_manager):
    with patch('torch.distributed.destroy_process_group') as mock_cleanup:
        model_manager.cleanup()
        mock_cleanup.assert_called_once()

@pytest.mark.asyncio
async def test_model_caching(model_manager):
    """Test that models are properly cached and reused."""
    mock_model = MagicMock()
    mock_model.config.hidden_size = 768

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_model) as mock_load:
        # First load should create new model
        model1 = await model_manager.get_model('test-model', 'sentiment')
        assert mock_load.call_count == 1

        # Second load should reuse cached model
        model2 = await model_manager.get_model('test-model', 'sentiment')
        assert mock_load.call_count == 1
        assert model1 is model2

@pytest.mark.asyncio
async def test_model_parallel_threshold(model_manager):
    """Test model parallelism threshold behavior."""
    # Test with small model
    mock_small = MagicMock()
    mock_small.config.hidden_size = 512  # Below threshold

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_small), \
         patch('torch.nn.DataParallel') as mock_data_parallel, \
         patch('torch.nn.parallel.DistributedDataParallel') as mock_distributed:

        await model_manager.get_model('small-model', 'sentiment', use_parallel=True)
        mock_data_parallel.assert_called_once()
        mock_distributed.assert_not_called()

    # Test with large model
    mock_large = MagicMock()
    mock_large.config.hidden_size = 1024  # Above threshold

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_large), \
         patch('torch.nn.DataParallel') as mock_data_parallel, \
         patch('torch.nn.parallel.DistributedDataParallel') as mock_distributed:

        await model_manager.get_model('large-model', 'sentiment', use_parallel=True)
        mock_distributed.assert_called_once()
        mock_data_parallel.assert_not_called()

@pytest.mark.asyncio
async def test_process_batch_with_parallel_model(model_manager):
    """Test batch processing with parallel model."""
    mock_model = MagicMock()
    mock_model.__class__ = torch.nn.DataParallel
    mock_results = [
        {'label': 'POSITIVE', 'score': 0.9},
        {'label': 'NEGATIVE', 'score': 0.8}
    ]

    with patch('transformers.pipeline', return_value=lambda x: mock_results):
        results = await model_manager.process_batch(
            mock_model,
            ['text1', 'text2'],
            'sentiment'
        )
        assert len(results) == 2
        assert all(isinstance(r, dict) for r in results)

@pytest.mark.asyncio
async def test_process_batch_error_handling(model_manager):
    """Test error handling during batch processing."""
    mock_model = MagicMock()

    def mock_pipeline_error(*args, **kwargs):
        raise RuntimeError("Pipeline error")

    with patch('transformers.pipeline', side_effect=mock_pipeline_error):
        with pytest.raises(RuntimeError, match="Pipeline error"):
            await model_manager.process_batch(
                mock_model,
                ['test text'],
                'sentiment'
            )

@pytest.mark.asyncio
async def test_model_gpu_placement(model_manager):
    """Test that models are placed on the correct GPU."""
    mock_model = MagicMock()
    mock_model.config.hidden_size = 768

    with patch('transformers.AutoModelForSequenceClassification.from_pretrained', return_value=mock_model) as mock_load:
        # Load first model
        model1 = await model_manager.get_model('model1', 'sentiment')
        assert model_manager.models['model1']['gpu_id'] == 0

        # Load second model
        model2 = await model_manager.get_model('model2', 'sentiment')
        assert model_manager.models['model2']['gpu_id'] == 1

        # Third model should wrap around to first GPU
        model3 = await model_manager.get_model('model3', 'sentiment')
        assert model_manager.models['model3']['gpu_id'] == 0
