# Use the official NVIDIA NGC TensorFlow container as a base image
FROM nvcr.io/nvidia/pytorch:23.06-py3

# Set the working directory
WORKDIR /app

# Copy the training service code into the container
COPY service.py .

# Install any additional dependencies
RUN pip install --no-cache-dir pytest

# Define the entry point for the container
ENTRYPOINT ["python", "service.py"]