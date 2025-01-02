import logging
from typing import Dict, Any, Optional
from dataclasses import dataclass
from datetime import datetime
import numpy as np
from cachetools import TTLCache, LRUCache
import psutil
import torch
import threading

logger = logging.getLogger(__name__)

@dataclass
class CacheStats:
    hits: int = 0
    misses: int = 0
    evictions: int = 0
    memory_usage: float = 0.0  # in MB

class MemoryAwareLRUCache:
    def __init__(self, maxsize: int, maxmemory_mb: float = 1024):
        self.maxmemory_mb = maxmemory_mb
        self.cache = LRUCache(maxsize=maxsize)
        self.memory_usage: Dict[str, float] = {}  # Track memory usage per key
        self.stats = CacheStats()
        self.lock = threading.Lock()

    def get(self, key: str) -> Optional[Any]:
        with self.lock:
            try:
                value = self.cache.get(key)
                if value is not None:
                    self.stats.hits += 1
                    return value
                self.stats.misses += 1
                return None
            except Exception as e:
                logger.error(f"Cache get error: {e}")
                return None

    def set(self, key: str, value: Any) -> None:
        with self.lock:
            try:
                # Estimate memory usage of the value
                if isinstance(value, np.ndarray):
                    memory_mb = value.nbytes / (1024 * 1024)
                elif isinstance(value, torch.Tensor):
                    memory_mb = value.element_size() * value.nelement() / (1024 * 1024)
                else:
                    # Fallback for other types
                    memory_mb = sys.getsizeof(value) / (1024 * 1024)

                # Check if adding this item would exceed memory limit
                current_memory = sum(self.memory_usage.values())
                if current_memory + memory_mb > self.maxmemory_mb:
                    self._evict_until_fits(memory_mb)

                # Add to cache
                self.cache[key] = value
                self.memory_usage[key] = memory_mb

            except Exception as e:
                logger.error(f"Cache set error: {e}")

    def _evict_until_fits(self, required_memory_mb: float) -> None:
        """Evict items until there's enough space for the new item."""
        while sum(self.memory_usage.values()) + required_memory_mb > self.maxmemory_mb:
            try:
                # Get the least recently used key
                lru_key = next(iter(self.cache))
                self.cache.pop(lru_key)
                self.memory_usage.pop(lru_key)
                self.stats.evictions += 1
            except (StopIteration, KeyError):
                break

    def get_stats(self) -> CacheStats:
        with self.lock:
            self.stats.memory_usage = sum(self.memory_usage.values())
            return self.stats

class InferenceService:
    def __init__(self, model_path: str, cache_size: int = 1000, cache_memory_mb: float = 1024):
        self.model = self._load_model(model_path)
        self.cache = MemoryAwareLRUCache(maxsize=cache_size, maxmemory_mb=cache_memory_mb)
        self._setup_monitoring()
        logger.info(f"Initialized InferenceService with cache size {cache_size} and memory limit {cache_memory_mb}MB")

    def _load_model(self, model_path: str):
        try:
            model = torch.load(model_path)
            model.eval()  # Set to evaluation mode
            if torch.cuda.is_available():
                model = model.cuda()
            return model
        except Exception as e:
            logger.error(f"Failed to load model: {e}")
            raise

    def _setup_monitoring(self):
        # Set up monitoring metrics
        from prometheus_client import Counter, Gauge
        self.prediction_counter = Counter('inference_predictions_total', 'Total number of predictions made')
        self.cache_hit_ratio = Gauge('inference_cache_hit_ratio', 'Cache hit ratio')
        self.cache_memory_usage = Gauge('inference_cache_memory_mb', 'Cache memory usage in MB')
        self.gpu_memory_usage = Gauge('inference_gpu_memory_mb', 'GPU memory usage in MB')

    async def predict(self, data: np.ndarray, cache_key: Optional[str] = None) -> np.ndarray:
        """
        Make a prediction with caching support.
        
        Args:
            data: Input data for prediction
            cache_key: Optional cache key. If None, caching is disabled for this prediction.
        
        Returns:
            Model prediction
        """
        try:
            # Try to get from cache if cache_key is provided
            if cache_key is not None:
                cached_result = self.cache.get(cache_key)
                if cached_result is not None:
                    return cached_result

            # Convert to tensor and move to GPU if available
            with torch.no_grad():
                if torch.cuda.is_available():
                    tensor_data = torch.from_numpy(data).cuda()
                else:
                    tensor_data = torch.from_numpy(data)

                # Make prediction
                prediction = self.model(tensor_data)
                prediction_np = prediction.cpu().numpy()

                # Cache result if cache_key is provided
                if cache_key is not None:
                    self.cache.set(cache_key, prediction_np)

                # Update metrics
                self._update_metrics()

                return prediction_np

        except Exception as e:
            logger.error(f"Prediction error: {e}")
            raise

    def _update_metrics(self):
        """Update monitoring metrics."""
        try:
            # Update prediction counter
            self.prediction_counter.inc()

            # Update cache metrics
            stats = self.cache.get_stats()
            total_requests = stats.hits + stats.misses
            if total_requests > 0:
                hit_ratio = stats.hits / total_requests
                self.cache_hit_ratio.set(hit_ratio)
            self.cache_memory_usage.set(stats.memory_usage)

            # Update GPU memory metrics if available
            if torch.cuda.is_available():
                gpu_memory_mb = torch.cuda.memory_allocated() / (1024 * 1024)
                self.gpu_memory_usage.set(gpu_memory_mb)

        except Exception as e:
            logger.error(f"Error updating metrics: {e}")

    def cleanup(self):
        """Clean up resources."""
        try:
            # Clear cache
            self.cache = None

            # Clear CUDA cache if using GPU
            if torch.cuda.is_available():
                torch.cuda.empty_cache()

            logger.info("Successfully cleaned up InferenceService resources")
        except Exception as e:
            logger.error(f"Error during cleanup: {e}") 