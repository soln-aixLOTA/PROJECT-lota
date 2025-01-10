#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "LotaBots Platform - Deployment"
echo "============================="

# Default values
NAMESPACE="lotabots"
REGISTRY="ghcr.io"
VERSION=$(git describe --tags --always)

# Function to check prerequisites
check_prerequisites() {
    # Check if kubectl is installed
    if ! command -v kubectl >/dev/null 2>&1; then
        echo -e "${RED}Error: kubectl not found${NC}"
        echo "Please install kubectl first"
        exit 1
    fi
    
    # Check if docker is installed
    if ! command -v docker >/dev/null 2>&1; then
        echo -e "${RED}Error: docker not found${NC}"
        echo "Please install docker first"
        exit 1
    fi
}

# Function to build and push Docker images
build_and_push() {
    local service=$1
    local dockerfile="src/$service/Dockerfile"
    
    if [ ! -f "$dockerfile" ]; then
        echo -e "${RED}Error: Dockerfile not found for $service${NC}"
        return 1
    fi
    
    echo "Building $service image..."
    docker build -t "$REGISTRY/$NAMESPACE/$service:$VERSION" -f "$dockerfile" .
    
    echo "Pushing $service image..."
    docker push "$REGISTRY/$NAMESPACE/$service:$VERSION"
    
    echo -e "${GREEN}✓ Image for $service built and pushed${NC}"
}

# Function to deploy Kubernetes manifests
deploy_manifests() {
    echo "Deploying Kubernetes manifests..."
    
    # Create namespace if it doesn't exist
    kubectl create namespace "$NAMESPACE" 2>/dev/null || true
    
    # Apply common resources
    kubectl apply -f k8s/common/ -n "$NAMESPACE"
    
    # Deploy each service
    for service in "attestation" "api_gateway"; do
        if [ -d "k8s/$service" ]; then
            echo "Deploying $service..."
            
            # Replace image tag in deployment manifest
            sed "s|IMAGE_TAG|$VERSION|g" "k8s/$service/deployment.yaml" | \
                kubectl apply -f - -n "$NAMESPACE"
            
            # Apply other resources
            find "k8s/$service" -name "*.yaml" ! -name "deployment.yaml" -exec \
                kubectl apply -f {} -n "$NAMESPACE" \;
        fi
    done
}

# Function to verify deployment
verify_deployment() {
    echo "Verifying deployment..."
    
    # Wait for deployments to be ready
    kubectl wait --for=condition=available --timeout=300s \
        deployment --all -n "$NAMESPACE"
    
    # Check pod status
    kubectl get pods -n "$NAMESPACE"
    
    # Check service endpoints
    kubectl get services -n "$NAMESPACE"
    
    echo -e "${GREEN}✓ Deployment verification completed${NC}"
}

# Function to rollback deployment
rollback_deployment() {
    echo -e "${YELLOW}Rolling back deployment...${NC}"
    
    for service in "attestation" "api_gateway"; do
        if kubectl get deployment "$service" -n "$NAMESPACE" >/dev/null 2>&1; then
            kubectl rollout undo deployment/"$service" -n "$NAMESPACE"
        fi
    done
    
    echo -e "${GREEN}✓ Rollback completed${NC}"
}

# Main deployment process
main() {
    # Check prerequisites
    check_prerequisites
    
    # Build and push images
    echo "Building and pushing Docker images..."
    for service in "attestation" "api_gateway"; do
        if ! build_and_push "$service"; then
            echo -e "${RED}Failed to build $service${NC}"
            exit 1
        fi
    done
    
    # Deploy to Kubernetes
    if ! deploy_manifests; then
        echo -e "${RED}Deployment failed${NC}"
        read -p "Do you want to rollback? (y/N) " should_rollback
        if [[ $should_rollback =~ ^[Yy]$ ]]; then
            rollback_deployment
        fi
        exit 1
    fi
    
    # Verify deployment
    if ! verify_deployment; then
        echo -e "${RED}Deployment verification failed${NC}"
        read -p "Do you want to rollback? (y/N) " should_rollback
        if [[ $should_rollback =~ ^[Yy]$ ]]; then
            rollback_deployment
        fi
        exit 1
    fi
    
    echo -e "\n${GREEN}Deployment completed successfully!${NC}"
    echo "Next steps:"
    echo "1. Monitor the application logs: kubectl logs -f -l app=lotabots -n $NAMESPACE"
    echo "2. Check the application status: kubectl get pods,services -n $NAMESPACE"
    echo "3. Access the application through the configured ingress or service"
}

main 