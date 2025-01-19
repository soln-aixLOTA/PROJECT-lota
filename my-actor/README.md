# GPU-Accelerated Web Scraping Actor

This Apify actor performs GPU-accelerated web scraping and content analysis using deep learning models.

## Features

- GPU-accelerated text processing with BERT models
- Image analysis with PyTorch
- Dynamic GPU resource management
- Prometheus metrics for monitoring
- Batched processing for efficient GPU utilization

## Requirements

- NVIDIA GPU with CUDA support
- Docker with NVIDIA Container Toolkit
- Python 3.11+

## Installation

1. Install the NVIDIA Container Toolkit:
```bash
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list
sudo apt-get update
sudo apt-get install -y nvidia-docker2
```

2. Build the Docker image:
```bash
docker build -t gpu-scraping-actor .
```

## Usage

1. Run the actor locally:
```bash
docker run --gpus all -p 9091:9091 gpu-scraping-actor
```

2. Monitor metrics:
- Access Prometheus metrics at `http://localhost:9091`

## Input

```json
{
    "url": "https://example.com"
}
```

## Output

The actor saves the following data:
- Text sentiment analysis
- Named entity recognition
- Image analysis statistics

## Monitoring

The actor exposes the following Prometheus metrics:
- `processed_texts_total`: Number of processed text items
- `processed_images_total`: Number of processed images
- `gpu_utilization_percent`: GPU utilization percentage
- `gpu_memory_used_mb`: GPU memory usage
- `processing_time_seconds`: Processing time for different tasks

## License

MIT
