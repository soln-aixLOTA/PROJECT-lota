#!/bin/bash

# Exit on error
set -e

echo "Deploying NVIDIA Inference Microservices (NIM)..."

# Create namespace if it doesn't exist
kubectl create namespace lotabots --dry-run=client -o yaml | kubectl apply -f -

# Apply configurations
echo "Applying configurations..."
kubectl apply -f k8s/base/nim-config.yaml
kubectl apply -f k8s/base/nim-secrets.yaml

# Apply network policies
echo "Applying network policies..."
kubectl apply -f k8s/base/nim-network-policy.yaml

# Deploy services
echo "Deploying NIM services..."
kubectl apply -f k8s/base/nim-deployment.yaml

# Wait for deployments to be ready
echo "Waiting for deployments to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/nim-embedding-service -n lotabots
kubectl wait --for=condition=available --timeout=300s deployment/nim-rerank-service -n lotabots
kubectl wait --for=condition=available --timeout=300s deployment/nim-llm-service -n lotabots

# Check pod status
echo "Checking pod status..."
kubectl get pods -n lotabots -l 'app in (nim-embedding-service,nim-rerank-service,nim-llm-service)'

# Run tests
echo "Running tests..."
python3 tests/nim_services_test.py

# Check services
echo "Checking service endpoints..."
kubectl get svc -n lotabots -l 'app in (nim-embedding-service,nim-rerank-service,nim-llm-service)'

# Monitor resources
echo "Monitoring GPU resources..."
nvidia-smi

echo "Deployment complete! Services are ready to use."
echo "Endpoints:"
echo "- Embedding Service: http://nim-embedding-service:8001/v1"
echo "- Rerank Service: http://nim-rerank-service:8002/v1"
echo "- LLM Service: http://nim-llm-service:8000/v1" 