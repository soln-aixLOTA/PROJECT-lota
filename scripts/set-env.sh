#!/bin/bash

# Source common functions
source ./lib/common.sh

# Default environment
ENV="dev"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --env)
            ENV="$2"
            shift 2
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Load environment-specific configuration
ENV_FILE="../config/${ENV}/.env"
if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
else
    log_warn "Environment file not found: $ENV_FILE"
    log_info "Using default environment variables"
fi

# Export environment variables
export PORT="${PORT:-8080}"
export DB_USER="${DB_USER:-dev_user}"
export DB_PASSWORD="${DB_PASSWORD:-dev_password}"
export DB_NAME="${DB_NAME:-document_automation_${ENV}}"
export DB_PORT="${DB_PORT:-5432}"

# JWT Secret handling
if [ -z "$JWT_SECRET" ]; then
    log_error "JWT_SECRET environment variable is not set!"
    log_error "For development: Run 'openssl rand -hex 32' and set the output as JWT_SECRET"
    log_error "For production: Use a secure secret management solution (HashiCorp Vault, AWS Secrets Manager, etc.)"
    exit 1
fi
export JWT_SECRET

export GRAFANA_PASSWORD="${GRAFANA_PASSWORD:-admin}"
export RUST_LOG="${RUST_LOG:-info}"
