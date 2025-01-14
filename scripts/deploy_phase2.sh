#!/bin/bash

set -e

# Function to wait for deployment rollout
wait_for_rollout() {
    local deployment=$1
    local namespace=$2
    echo "Waiting for deployment $deployment to complete..."
    kubectl rollout status deployment/$deployment -n $namespace --timeout=300s
}

# Function to wait for StatefulSet rollout
wait_for_statefulset() {
    local statefulset=$1
    local namespace=$2
    echo "Waiting for StatefulSet $statefulset to complete..."
    kubectl rollout status statefulset/$statefulset -n $namespace --timeout=300s
}

# Create Redis password secret
echo "Creating Redis secret..."
REDIS_PASSWORD=$(openssl rand -base64 32)
kubectl create secret generic redis-secret \
    --from-literal=password=$REDIS_PASSWORD \
    -n lotabots \
    --dry-run=client -o yaml | kubectl apply -f -

# Deploy Redis
echo "Deploying Redis..."
kubectl apply -f k8s/base/redis/deployment.yaml
wait_for_statefulset redis lotabots

# Create Redis ConfigMap
echo "Creating Redis ConfigMap..."
kubectl create configmap redis-config \
    --from-file=redis.conf=k8s/base/redis/redis.conf \
    -n lotabots \
    --dry-run=client -o yaml | kubectl apply -f -

# Update API Gateway configuration
echo "Updating API Gateway configuration..."
kubectl create configmap api-gateway-config \
    --from-literal=REDIS_URL="redis://redis:6379" \
    --from-literal=REDIS_PASSWORD=$REDIS_PASSWORD \
    -n lotabots \
    --dry-run=client -o yaml | kubectl apply -f -

# Rebuild and deploy API Gateway
echo "Building new API Gateway image..."
docker build -t lotabots/api-gateway:latest src/api_gateway
docker push lotabots/api-gateway:latest

echo "Deploying updated API Gateway..."
kubectl apply -f k8s/base/api-gateway/deployment.yaml
wait_for_rollout api-gateway lotabots

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 30

# Run basic health checks
echo "Running health checks..."
kubectl exec -n lotabots deploy/api-gateway -- curl -s http://localhost:8080/health
kubectl exec -n lotabots statefulset/redis -- redis-cli -a $REDIS_PASSWORD ping

echo "Phase 2 deployment completed successfully!"
echo "Please verify the following:"
echo "1. Redis cluster is operational"
echo "2. API Gateway can connect to Redis"
echo "3. Rate limiting is working as expected"
echo "4. Cache hit rates are being monitored"
echo "5. All services are healthy"

# Display Redis metrics
echo "Redis metrics:"
kubectl exec -n lotabots statefulset/redis -- redis-cli -a $REDIS_PASSWORD info stats | grep -E "keyspace|hits|misses"

# Display API Gateway metrics
echo "API Gateway metrics:"
kubectl exec -n lotabots deploy/api-gateway -- curl -s localhost:9090/metrics | grep -E "rate_limit|cache" 