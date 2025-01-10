#!/bin/bash

set -e

# Function to wait for deployment rollout
wait_for_rollout() {
    local deployment=$1
    local namespace=$2
    echo "Waiting for deployment $deployment to complete..."
    kubectl rollout status deployment/$deployment -n $namespace --timeout=300s
}

# Function to check if a namespace exists
ensure_namespace() {
    local namespace=$1
    if ! kubectl get namespace $namespace &> /dev/null; then
        echo "Creating namespace $namespace..."
        kubectl create namespace $namespace
    fi
}

# Setup environment
echo "Setting up environment..."
ensure_namespace lotabots

# Deploy Vault
echo "Deploying HashiCorp Vault..."
./scripts/setup_vault.sh

# Wait for Vault to be ready
echo "Waiting for Vault to be ready..."
kubectl wait --for=condition=ready pod -l app=vault -n lotabots --timeout=300s

# Build and push API Gateway image
echo "Building API Gateway image..."
docker build -t lotabots/api-gateway:latest src/api_gateway
docker push lotabots/api-gateway:latest

# Deploy API Gateway
echo "Deploying API Gateway..."
kubectl apply -f k8s/base/api-gateway/deployment.yaml
wait_for_rollout api-gateway lotabots

# Update Inference Service to use Vault
echo "Updating Inference Service..."
kubectl apply -f k8s/base/inference-service/deployment.yaml
wait_for_rollout inference-service lotabots

# Migrate existing secrets
echo "Migrating existing secrets to Vault..."
python3 scripts/migrate_secrets.py

echo "Phase 1 deployment completed successfully!"
echo "Please verify the following:"
echo "1. Vault is operational and accessible"
echo "2. API Gateway is handling requests with improved concurrency"
echo "3. Inference Service is using Vault for secrets"
echo "4. All services are healthy in Kubernetes dashboard"

# Optional: Run basic health check
echo "Running basic health check..."
kubectl exec -n lotabots deploy/api-gateway -- curl -s http://localhost:8080/health
kubectl exec -n lotabots deploy/inference-service -- curl -s http://localhost:50051/health 