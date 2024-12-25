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

# Create Grafana admin password secret
echo "Creating Grafana secret..."
GRAFANA_PASSWORD=$(openssl rand -base64 32)
kubectl create secret generic grafana-secret \
    --from-literal=admin-password=$GRAFANA_PASSWORD \
    -n lotabots \
    --dry-run=client -o yaml | kubectl apply -f -

# Deploy Prometheus
echo "Deploying Prometheus..."
kubectl apply -f k8s/base/monitoring/prometheus-config.yaml
kubectl apply -f k8s/base/monitoring/prometheus-deployment.yaml
wait_for_statefulset prometheus lotabots

# Deploy Grafana
echo "Deploying Grafana..."
kubectl apply -f k8s/base/monitoring/grafana-deployment.yaml
wait_for_rollout grafana lotabots

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 30

# Run basic health checks
echo "Running health checks..."
kubectl exec -n lotabots deploy/grafana -- curl -s http://localhost:3000/api/health
kubectl exec -n lotabots statefulset/prometheus -- curl -s http://localhost:9090/-/healthy

echo "Phase 3 deployment completed successfully!"
echo "Please verify the following:"
echo "1. Prometheus is collecting metrics"
echo "2. Grafana dashboards are accessible"
echo "3. Alert rules are properly configured"
echo "4. All services are being monitored"

# Display access information
echo -e "\nAccess Information:"
echo "Grafana:"
echo "  URL: http://grafana.lotabots.svc:3000"
echo "  Username: admin"
echo "  Password: $GRAFANA_PASSWORD"
echo "Prometheus:"
echo "  URL: http://prometheus.lotabots.svc:9090"

# Check Prometheus targets
echo -e "\nChecking Prometheus targets..."
kubectl exec -n lotabots statefulset/prometheus -- curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job:.labels.job, health:.health}'

# Check Prometheus alerts
echo -e "\nChecking Prometheus alerts..."
kubectl exec -n lotabots statefulset/prometheus -- curl -s http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | {name:.labels.alertname, state:.state}'

# Display some initial metrics
echo -e "\nInitial metrics:"
echo "API Gateway request rate:"
kubectl exec -n lotabots statefulset/prometheus -- curl -s 'http://localhost:9090/api/v1/query' --data-urlencode 'query=sum(rate(http_requests_total[5m]))' | jq '.data.result[0].value[1]'

echo "Inference Service GPU utilization:"
kubectl exec -n lotabots statefulset/prometheus -- curl -s 'http://localhost:9090/api/v1/query' --data-urlencode 'query=avg(nvidia_gpu_duty_cycle)' | jq '.data.result[0].value[1]' 