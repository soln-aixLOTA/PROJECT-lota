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
        -h|--help)
            echo "Usage: $0 [options]"
            echo
            echo "Options:"
            echo "  --env ENV     Environment to use: dev|test|prod (default: dev)"
            echo "  -h, --help    Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate environment
case "$ENV" in
    dev|test|prod)
        ;;
    *)
        log_error "Invalid environment: $ENV"
        log_info "Valid environments are: dev, test, prod"
        exit 1
        ;;
esac

# Load environment configuration
CONFIG_FILE="config/$ENV/config.sh"
if [ ! -f "$CONFIG_FILE" ]; then
    log_error "Configuration file not found: $CONFIG_FILE"
    exit 1
fi

# Source environment configuration
source "$CONFIG_FILE"

# Export environment name
export ENVIRONMENT="$ENV"

log_info "Environment set to: $ENV"
log_info "Service URL: $SERVICE_URL"
log_info "Test mode: $TEST_MODE" 