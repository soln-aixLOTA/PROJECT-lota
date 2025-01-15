import pytest
from PIL import Image
import numpy as np
from src.main import ProcessingQueue

@pytest.fixture
def queue():
    return ProcessingQueue(batch_size=2)

@pytest.fixture
def sample_image():
    # Create a small test image
    return Image.fromarray(np.zeros((64, 64, 3), dtype=np.uint8))

def test_add_text(queue):
    queue.add_text("test text 1")
    assert len(queue.text_queue) == 1
    assert queue.text_queue[0] == "test text 1"

def test_add_image(queue, sample_image):
    queue.add_image(sample_image)
    assert len(queue.image_queue) == 1
    assert queue.image_queue[0] == sample_image

def test_get_text_batch_empty(queue):
    assert queue.get_text_batch() == []

def test_get_text_batch_partial(queue):
    queue.add_text("test text 1")
    assert queue.get_text_batch() == []  # Not enough items for a batch

def test_get_text_batch_full(queue):
    queue.add_text("test text 1")
    queue.add_text("test text 2")
    batch = queue.get_text_batch()
    assert len(batch) == 2
    assert batch == ["test text 1", "test text 2"]
    assert len(queue.text_queue) == 0

def test_get_text_batch_overflow(queue):
    queue.add_text("test text 1")
    queue.add_text("test text 2")
    queue.add_text("test text 3")
    batch = queue.get_text_batch()
    assert len(batch) == 2
    assert batch == ["test text 1", "test text 2"]
    assert len(queue.text_queue) == 1
    assert queue.text_queue[0] == "test text 3"

def test_get_image_batch_empty(queue):
    assert queue.get_image_batch() == []

def test_get_image_batch_partial(queue, sample_image):
    queue.add_image(sample_image)
    assert queue.get_image_batch() == []  # Not enough items for a batch

def test_get_image_batch_full(queue, sample_image):
    queue.add_image(sample_image)
    queue.add_image(sample_image)
    batch = queue.get_image_batch()
    assert len(batch) == 2
    assert all(isinstance(img, Image.Image) for img in batch)
    assert len(queue.image_queue) == 0

def test_get_image_batch_overflow(queue, sample_image):
    queue.add_image(sample_image)
    queue.add_image(sample_image)
    queue.add_image(sample_image)
    batch = queue.get_image_batch()
    assert len(batch) == 2
    assert all(isinstance(img, Image.Image) for img in batch)
    assert len(queue.image_queue) == 1

def test_custom_batch_size():
    """Test queue with custom batch size."""
    queue = ProcessingQueue(batch_size=3)

    # Add 4 items
    for i in range(4):
        queue.add_text(f"text {i}")

    # Should get batch of 3
    batch = queue.get_text_batch()
    assert len(batch) == 3
    assert batch == ["text 0", "text 1", "text 2"]

    # One item should remain
    assert len(queue.text_queue) == 1
    assert queue.text_queue[0] == "text 3"

def test_mixed_queue_operations():
    """Test interleaved text and image queue operations."""
    queue = ProcessingQueue(batch_size=2)
    sample_image = Image.fromarray(np.zeros((64, 64, 3), dtype=np.uint8))

    # Add items alternately
    queue.add_text("text 1")
    queue.add_image(sample_image)
    queue.add_text("text 2")
    queue.add_image(sample_image)

    # Check text batch
    text_batch = queue.get_text_batch()
    assert len(text_batch) == 2
    assert text_batch == ["text 1", "text 2"]

    # Check image batch
    image_batch = queue.get_image_batch()
    assert len(image_batch) == 2
    assert all(isinstance(img, Image.Image) for img in image_batch)

def test_queue_stress():
    """Stress test the queue with many operations."""
    queue = ProcessingQueue(batch_size=10)
    sample_image = Image.fromarray(np.zeros((64, 64, 3), dtype=np.uint8))

    # Add many items
    for i in range(100):
        if i % 2 == 0:
            queue.add_text(f"text {i}")
        else:
            queue.add_image(sample_image)

    # Process all text batches
    text_batches = []
    while True:
        batch = queue.get_text_batch()
        if not batch:
            break
        text_batches.append(batch)

    # Process all image batches
    image_batches = []
    while True:
        batch = queue.get_image_batch()
        if not batch:
            break
        image_batches.append(batch)

    # Verify results
    assert len(text_batches) == 5  # 50 texts / batch_size 10
    assert len(image_batches) == 5  # 50 images / batch_size 10
    assert all(len(batch) == 10 for batch in text_batches)
    assert all(len(batch) == 10 for batch in image_batches)

def test_empty_queue_behavior():
    """Test behavior when getting batches from empty queue."""
    queue = ProcessingQueue(batch_size=2)

    # Try getting batches from empty queue
    assert queue.get_text_batch() == []
    assert queue.get_image_batch() == []

    # Add single items and verify no premature batching
    queue.add_text("text 1")
    assert queue.get_text_batch() == []

    queue.add_image(Image.fromarray(np.zeros((64, 64, 3), dtype=np.uint8)))
    assert queue.get_image_batch() == []

def test_invalid_inputs():
    """Test handling of invalid inputs."""
    queue = ProcessingQueue(batch_size=2)

    # Test invalid text
    with pytest.raises(AttributeError):
        queue.add_text(None)

    # Test invalid image
    with pytest.raises(AttributeError):
        queue.add_image("not an image")

    # Verify queue state wasn't corrupted
    assert len(queue.text_queue) == 0
    assert len(queue.image_queue) == 0

def test_batch_size_validation():
    """Test validation of batch size parameter."""
    # Test invalid batch sizes
    with pytest.raises(ValueError):
        ProcessingQueue(batch_size=0)

    with pytest.raises(ValueError):
        ProcessingQueue(batch_size=-1)

    # Test valid batch sizes
    queue1 = ProcessingQueue(batch_size=1)
    assert queue1.batch_size == 1

    queue100 = ProcessingQueue(batch_size=100)
    assert queue100.batch_size == 100
