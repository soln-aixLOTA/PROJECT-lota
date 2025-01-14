# Use an NVIDIA AI Enterprise or RAPIDS-based container from NGC
FROM nvcr.io/nvidia/rapidsai/rapidsai:23.06-cuda11.4-runtime-ubuntu20.04-py3.10

WORKDIR /app
COPY service.py .
RUN pip install --no-cache-dir pytest  # For testing

ENTRYPOINT ["python", "service.py"] 