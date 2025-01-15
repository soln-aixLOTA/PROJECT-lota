import os
import json
import logging
import time
from datetime import datetime
import torch
import torch.nn as nn
import numpy as np
from prometheus_client import start_http_server, Gauge, Counter

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('benchmark_agent')

# Prometheus metrics
GPU_UTILIZATION = Gauge('gpu_utilization', 'GPU utilization percentage')
GPU_MEMORY_USAGE = Gauge('gpu_memory_usage', 'GPU memory usage in MB')
BENCHMARK_DURATION = Gauge('benchmark_duration', 'Benchmark duration in seconds')
OPERATIONS_COUNTER = Counter('operations_total', 'Total number of operations performed')

class BenchmarkAgent:
    def __init__(self):
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.info(f"Using device: {self.device}")

        # Output directory
        self.output_dir = "/app/shared_data"
        os.makedirs(self.output_dir, exist_ok=True)

        # Start Prometheus metrics server
        start_http_server(8000)

    def run_benchmarks(self):
        """Run various performance benchmarks."""
        logger.info("Starting performance benchmarks")

        try:
            benchmark_results = {
                "timestamp": datetime.now().isoformat(),
                "device_info": self.get_device_info(),
                "benchmarks": {
                    "matrix_operations": self.benchmark_matrix_operations(),
                    "model_inference": self.benchmark_model_inference(),
                    "memory_bandwidth": self.benchmark_memory_bandwidth()
                }
            }

            # Save results
            output_file = os.path.join(self.output_dir, "benchmark_results.json")
            with open(output_file, 'w') as f:
                json.dump(benchmark_results, f, indent=2)

            logger.info(f"Benchmarks completed and saved to {output_file}")
            return benchmark_results

        except Exception as e:
            logger.error(f"Error running benchmarks: {str(e)}")
            return None

    def get_device_info(self):
        """Get information about the compute device."""
        if self.device.type == "cuda":
            return {
                "device_type": "GPU",
                "device_name": torch.cuda.get_device_name(0),
                "compute_capability": f"{torch.cuda.get_device_capability()}",
                "total_memory": f"{torch.cuda.get_device_properties(0).total_memory / 1024**2:.2f} MB",
                "cuda_version": torch.version.cuda
            }
        else:
            return {
                "device_type": "CPU",
                "device_name": "CPU"
            }

    def benchmark_matrix_operations(self):
        """Benchmark matrix multiplication performance."""
        logger.info("Running matrix operations benchmark")

        sizes = [1000, 2000, 4000]
        results = {}

        for size in sizes:
            # Create random matrices
            matrix1 = torch.randn(size, size, device=self.device)
            matrix2 = torch.randn(size, size, device=self.device)

            # Warm-up
            torch.matmul(matrix1, matrix2)
            torch.cuda.synchronize()

            # Benchmark
            start_time = time.time()
            for _ in range(10):
                torch.matmul(matrix1, matrix2)
                OPERATIONS_COUNTER.inc()
            torch.cuda.synchronize()
            end_time = time.time()

            results[f"{size}x{size}"] = {
                "time_per_operation": (end_time - start_time) / 10,
                "operations_per_second": 10 / (end_time - start_time)
            }

            # Update metrics
            if self.device.type == "cuda":
                GPU_UTILIZATION.set(torch.cuda.utilization())
                GPU_MEMORY_USAGE.set(torch.cuda.memory_allocated() / 1024**2)

        return results

    def benchmark_model_inference(self):
        """Benchmark model inference performance."""
        logger.info("Running model inference benchmark")

        # Create a simple model
        model = nn.Sequential(
            nn.Linear(1000, 512),
            nn.ReLU(),
            nn.Linear(512, 256),
            nn.ReLU(),
            nn.Linear(256, 10)
        ).to(self.device)

        batch_sizes = [1, 8, 32, 128]
        results = {}

        for batch_size in batch_sizes:
            # Create random input
            input_data = torch.randn(batch_size, 1000, device=self.device)

            # Warm-up
            model(input_data)
            torch.cuda.synchronize()

            # Benchmark
            start_time = time.time()
            for _ in range(100):
                model(input_data)
                OPERATIONS_COUNTER.inc()
            torch.cuda.synchronize()
            end_time = time.time()

            results[f"batch_size_{batch_size}"] = {
                "time_per_inference": (end_time - start_time) / 100,
                "inferences_per_second": 100 / (end_time - start_time)
            }

        return results

    def benchmark_memory_bandwidth(self):
        """Benchmark memory bandwidth."""
        logger.info("Running memory bandwidth benchmark")

        sizes = [1024**2, 1024**3]  # 1MB, 1GB
        results = {}

        for size in sizes:
            # Create large tensor
            data = torch.randn(size, device=self.device)

            # Warm-up
            _ = torch.clone(data)
            torch.cuda.synchronize()

            # Benchmark
            start_time = time.time()
            for _ in range(10):
                _ = torch.clone(data)
                OPERATIONS_COUNTER.inc()
            torch.cuda.synchronize()
            end_time = time.time()

            bandwidth = (size * 4 * 10) / (end_time - start_time)  # bytes per second
            results[f"{size // 1024**2}MB"] = {
                "bandwidth_gbps": bandwidth / 1024**3,
                "time_per_operation": (end_time - start_time) / 10
            }

        return results

def main():
    agent = BenchmarkAgent()
    agent.run_benchmarks()

if __name__ == "__main__":
    main()
