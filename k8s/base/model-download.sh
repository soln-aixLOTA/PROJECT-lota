#!/bin/sh
set -e

# Create model directories
mkdir -p /models/{embedding,rerank,llm}

# Set up headers for NIM API
AUTH_HEADER="Authorization: Bearer ${NVIDIA_API_KEY}"
ACCEPT_HEADER="Accept: application/json"

# Download models
echo "Downloading models from NIM..."
cd /models

# Download embedding model
echo "Downloading embedding model..."
curl -X GET "https://api.nvcr.io/v2/nim/models/nvidia/nv-embedqa-e5-v5/versions/1.0.0/files" \
  -H "${AUTH_HEADER}" -H "${ACCEPT_HEADER}" | jq -r '.files[].downloadUrl' | \
  while read url; do
    curl -O -L -H "${AUTH_HEADER}" "$url"
  done
mv nv-embedqa-e5-v5* embedding/

# Download rerank model
echo "Downloading rerank model..."
curl -X GET "https://api.nvcr.io/v2/nim/models/nvidia/nv-rerankqa-mistral-4b-v3/versions/1.0.0/files" \
  -H "${AUTH_HEADER}" -H "${ACCEPT_HEADER}" | jq -r '.files[].downloadUrl' | \
  while read url; do
    curl -O -L -H "${AUTH_HEADER}" "$url"
  done
mv nv-rerankqa-mistral-4b-v3* rerank/

# Download LLM model
echo "Downloading LLM model..."
curl -X GET "https://api.nvcr.io/v2/nim/models/meta/llama3-8b-instruct/versions/1.0.0/files" \
  -H "${AUTH_HEADER}" -H "${ACCEPT_HEADER}" | jq -r '.files[].downloadUrl' | \
  while read url; do
    curl -O -L -H "${AUTH_HEADER}" "$url"
  done
mv llama3-8b-instruct* llm/

# Set permissions
echo "Setting permissions..."
chmod -R 755 /models

echo "Model download and organization complete" 