#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "LotaBots Platform - Environment Configuration"
echo "==========================================="

# Default values
DEFAULT_VAULT_ADDR="http://vault.lotabots.svc:8200"
DEFAULT_VAULT_ROLE="api-gateway"
DEFAULT_VAULT_NAMESPACE="lotabots"

# Function to prompt for value with default
prompt_with_default() {
    local prompt=$1
    local default=$2
    local value
    
    read -p "$prompt [$default]: " value
    echo "${value:-$default}"
}

# Create .env file
create_env_file() {
    local env_file=".env"
    
    echo "Creating .env file..."
    
    # Prompt for Vault configuration
    VAULT_ADDR=$(prompt_with_default "Enter Vault address" "$DEFAULT_VAULT_ADDR")
    VAULT_ROLE=$(prompt_with_default "Enter Vault role" "$DEFAULT_VAULT_ROLE")
    VAULT_NAMESPACE=$(prompt_with_default "Enter Vault namespace" "$DEFAULT_VAULT_NAMESPACE")
    
    # Prompt for database configuration
    DB_HOST=$(prompt_with_default "Enter PostgreSQL host" "localhost")
    DB_PORT=$(prompt_with_default "Enter PostgreSQL port" "5432")
    DB_NAME=$(prompt_with_default "Enter PostgreSQL database name" "lotabots")
    DB_USER=$(prompt_with_default "Enter PostgreSQL username" "postgres")
    read -s -p "Enter PostgreSQL password: " DB_PASSWORD
    echo
    
    # Prompt for Redis configuration
    REDIS_HOST=$(prompt_with_default "Enter Redis host" "localhost")
    REDIS_PORT=$(prompt_with_default "Enter Redis port" "6379")
    read -s -p "Enter Redis password (leave empty if none): " REDIS_PASSWORD
    echo
    
    # Write to .env file
    cat > "$env_file" << EOF
# Vault Configuration
VAULT_ADDR=$VAULT_ADDR
VAULT_ROLE=$VAULT_ROLE
VAULT_NAMESPACE=$VAULT_NAMESPACE

# Database Configuration
DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME

# Redis Configuration
REDIS_URL=redis://:${REDIS_PASSWORD}@$REDIS_HOST:$REDIS_PORT

# Logging Configuration
RUST_LOG=info
RUST_BACKTRACE=1

# OpenTelemetry Configuration
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=lotabots-api

# API Configuration
API_PORT=8080
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_DURATION=60
EOF

    chmod 600 "$env_file"
    echo -e "${GREEN}✓ Environment file created at .env${NC}"
}

# Create Kubernetes secrets
create_k8s_secrets() {
    echo "Creating Kubernetes secrets..."
    
    # Check if kubectl is available and configured
    if ! command -v kubectl >/dev/null 2>&1; then
        echo -e "${RED}kubectl not found. Please install kubectl first.${NC}"
        exit 1
    fi
    
    # Create namespace if it doesn't exist
    kubectl create namespace lotabots 2>/dev/null || true
    
    # Create secrets from .env file
    kubectl create secret generic lotabots-secrets \
        --from-file=.env \
        --namespace lotabots \
        --dry-run=client -o yaml | kubectl apply -f -
    
    echo -e "${GREEN}✓ Kubernetes secrets created${NC}"
}

# Main configuration process
main() {
    # Create environment file
    create_env_file
    
    # Ask if user wants to create Kubernetes secrets
    read -p "Do you want to create Kubernetes secrets? (y/N) " create_secrets
    if [[ $create_secrets =~ ^[Yy]$ ]]; then
        create_k8s_secrets
    fi
    
    echo -e "\n${GREEN}Environment configuration completed!${NC}"
    echo "Next steps:"
    echo "1. Review the .env file and adjust values if needed"
    echo "2. Make sure your Kubernetes cluster has access to the secrets"
    echo "3. Run the application with 'cargo run' or deploy to Kubernetes"
}

main 