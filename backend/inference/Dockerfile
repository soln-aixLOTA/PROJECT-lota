# Use the official NVIDIA NGC Triton Inference Server container as a base image
FROM nvcr.io/nvidia/pytriton:23.03-py3

# Set the working directory
WORKDIR /app

# Copy the inference service code into the container
COPY service.py .

# Install any additional dependencies
RUN pip install --no-cache-dir pytest

# Define the entry point for the container
ENTRYPOINT ["python", "service.py"]