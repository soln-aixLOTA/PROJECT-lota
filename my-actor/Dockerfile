# Use NVIDIA CUDA base image
FROM nvidia/cuda:12.3.2-runtime-ubuntu22.04

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    DEBIAN_FRONTEND=noninteractive \
    CUDA_HOME=/usr/local/cuda \
    PATH=/usr/local/cuda/bin:$PATH \
    LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.11 \
    python3-pip \
    python3-dev \
    git \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy requirements first for better caching
COPY requirements.txt ./

# Install Python dependencies
RUN pip3 install --no-cache-dir -r requirements.txt

# Copy actor files
COPY . .

# Create necessary directories
RUN mkdir -p /usr/src/app/storage/datasets/default \
    && mkdir -p /usr/src/app/storage/key_value_stores/default \
    && mkdir -p /usr/src/app/storage/request_queue/default

# Set environment variables for GPU support
ENV NVIDIA_VISIBLE_DEVICES=all \
    NVIDIA_DRIVER_CAPABILITIES=compute,utility \
    NVIDIA_REQUIRE_CUDA="cuda>=12.0"

# Expose Prometheus metrics port
EXPOSE 9091

# Run the actor
CMD ["python3", "src/main.py"]
