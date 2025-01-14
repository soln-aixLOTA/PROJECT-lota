#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Document Automation Platform - Deployment"
echo "======================================="

# Default values
DEPLOYMENT_TYPE="local"  # local or k8s
NAMESPACE="lotabots"
REGISTRY="ghcr.io"
VERSION=$(git describe --tags --always 2>/dev/null || echo "latest")

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --type     Deployment type (local or k8s) [default: local]"
    echo "  --help     Show this help message"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            DEPLOYMENT_TYPE="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Function to check prerequisites
check_prerequisites() {
    echo "Checking prerequisites..."

    # Check Docker
    if ! command -v docker >/dev/null 2>&1; then
        echo -e "${RED}Error: docker not found${NC}"
        echo "Please install docker first"
        exit 1
    fi

    # Check docker-compose for local deployment
    if [ "$DEPLOYMENT_TYPE" = "local" ] && ! command -v docker-compose >/dev/null 2>&1; then
        echo -e "${RED}Error: docker-compose not found${NC}"
        echo "Please install docker-compose first"
        exit 1
    fi

    # Check kubectl for k8s deployment
    if [ "$DEPLOYMENT_TYPE" = "k8s" ] && ! command -v kubectl >/dev/null 2>&1; then
        echo -e "${RED}Error: kubectl not found${NC}"
        echo "Please install kubectl first"
        exit 1
    fi

    # Check NVIDIA prerequisites for local deployment
    if [ "$DEPLOYMENT_TYPE" = "local" ]; then
        if ! command -v nvidia-smi &> /dev/null; then
            echo -e "${RED}Error: NVIDIA drivers not found${NC}"
            echo "Please install NVIDIA drivers first"
            exit 1
        fi

        if ! docker info | grep -i "nvidia" &> /dev/null; then
            echo -e "${RED}Error: NVIDIA Docker runtime not found${NC}"
            echo "Please install nvidia-docker2"
            exit 1
        fi
    fi
}

# Function to check environment variables
check_env() {
    echo "Checking environment variables..."

    if [ -z "$HUGGING_FACE_HUB_TOKEN" ]; then
        echo -e "${RED}Error: HUGGING_FACE_HUB_TOKEN not set${NC}"
        echo "Please set it in your environment or .env file"
        exit 1
    fi
}

# Function for local deployment
deploy_local() {
    echo "Starting local deployment..."

    echo "Building and starting services..."
    docker-compose build --no-cache
    docker-compose up -d

    echo "Waiting for services to start..."
    sleep 10

    echo "Running database migrations..."
    docker-compose exec -T app /usr/local/bin/document-automation migrate

    echo "Checking service health..."
    if curl -s http://localhost:3000/health > /dev/null; then
        echo -e "${GREEN}✓ Rust application is running${NC}"
    else
        echo -e "${RED}× Warning: Rust application health check failed${NC}"
    fi

    if curl -s http://localhost:8000/v1/models > /dev/null; then
        echo -e "${GREEN}✓ vLLM service is running${NC}"
    else
        echo -e "${RED}× Warning: vLLM service health check failed${NC}"
    fi

    echo -e "\n${GREEN}Local deployment complete!${NC}"
    echo "Services available at:"
    echo "- Rust Application: http://localhost:3000"
    echo "- vLLM Service: http://localhost:8000"
}

# Function for Kubernetes deployment
deploy_k8s() {
    echo "Starting Kubernetes deployment..."

    # Create namespace if it doesn't exist
    kubectl create namespace "$NAMESPACE" 2>/dev/null || true

    # Apply common resources
    kubectl apply -f k8s/common/ -n "$NAMESPACE"

    # Deploy each service
    for service in "attestation" "api_gateway"; do
        if [ -d "k8s/$service" ]; then
            echo "Deploying $service..."
            sed "s|IMAGE_TAG|$VERSION|g" "k8s/$service/deployment.yaml" | \
                kubectl apply -f - -n "$NAMESPACE"
            find "k8s/$service" -name "*.yaml" ! -name "deployment.yaml" -exec \
                kubectl apply -f {} -n "$NAMESPACE" \;
        fi
    done

    # Verify deployment
    echo "Verifying deployment..."
    kubectl wait --for=condition=available --timeout=300s \
        deployment --all -n "$NAMESPACE"

    kubectl get pods,services -n "$NAMESPACE"

    echo -e "\n${GREEN}Kubernetes deployment complete!${NC}"
    echo "Next steps:"
    echo "1. Monitor logs: kubectl logs -f -l app=lotabots -n $NAMESPACE"
    echo "2. Check status: kubectl get pods,services -n $NAMESPACE"
}

# Function to handle deployment failure
handle_failure() {
    echo -e "${RED}Deployment failed${NC}"

    if [ "$DEPLOYMENT_TYPE" = "local" ]; then
        echo "Cleaning up..."
        docker-compose down
    elif [ "$DEPLOYMENT_TYPE" = "k8s" ]; then
        read -p "Do you want to rollback? (y/N) " should_rollback
        if [[ $should_rollback =~ ^[Yy]$ ]]; then
            echo "Rolling back..."
            for service in "attestation" "api_gateway"; do
                kubectl rollout undo deployment/"$service" -n "$NAMESPACE" 2>/dev/null || true
            done
        fi
    fi

    exit 1
}

# Main deployment process
main() {
    check_prerequisites
    check_env

    # Set up error handling
    trap handle_failure ERR

    case "$DEPLOYMENT_TYPE" in
        "local")
            deploy_local
            ;;
        "k8s")
            deploy_k8s
            ;;
        *)
            echo -e "${RED}Error: Invalid deployment type${NC}"
            show_usage
            exit 1
            ;;
    esac
}

main
