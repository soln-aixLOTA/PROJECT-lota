#!/bin/bash

set -e

# Function to wait for deployment rollout
wait_for_rollout() {
    local deployment=$1
    local namespace=$2
    echo "Waiting for deployment $deployment to complete..."
    kubectl rollout status deployment/$deployment -n $namespace --timeout=300s
}

# Function to monitor GPU metrics
monitor_gpu_metrics() {
    local pod=$1
    local namespace=$2
    echo "Monitoring GPU metrics for pod $pod..."
    kubectl exec -n $namespace $pod -- nvidia-smi dmon -s pucvmet -o TD -c 1
}

# Apply Triton model configuration
echo "Applying Triton model configuration..."
kubectl create configmap triton-model-config -n lotabots \
    --from-file=model-config.pbtxt=k8s/base/inference-service/model-config.pbtxt \
    --dry-run=client -o yaml | kubectl apply -f -

# Update Inference Service deployment
echo "Updating Inference Service deployment..."
kubectl apply -f k8s/base/inference-service/deployment.yaml
wait_for_rollout inference-service lotabots

# Wait for pods to be ready
echo "Waiting for pods to be ready..."
sleep 30

# Get a pod for monitoring
POD=$(kubectl get pod -n lotabots -l app=inference-service -o jsonpath='{.items[0].metadata.name}')

# Monitor initial performance
echo "Monitoring initial performance..."
monitor_gpu_metrics $POD lotabots

# Apply load test (if hey is installed)
if command -v hey &> /dev/null; then
    echo "Running load test..."
    hey -z 30s -c 50 http://inference-service.lotabots.svc:8000/v2/models/gemini-2.0-flash-exp/infer
fi

# Monitor performance after load test
echo "Monitoring performance after load test..."
monitor_gpu_metrics $POD lotabots

# Check HPA status
echo "Checking HPA status..."
kubectl get hpa inference-service-hpa -n lotabots

# Display Triton metrics
echo "Displaying Triton metrics..."
kubectl exec -n lotabots $POD -- curl -s localhost:8002/metrics | grep -E "nv_inference|nv_gpu"

echo "Performance optimization complete!"
echo "Please monitor the following metrics in Grafana:"
echo "1. GPU utilization"
echo "2. Inference latency"
echo "3. Request throughput"
echo "4. Memory usage"
echo "5. Cache hit rate" 