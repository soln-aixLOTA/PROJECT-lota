#!/bin/bash

# Logging functions
log_info() {
    echo "[INFO] $1"
}

log_warn() {
    echo "[WARN] $1" >&2
}

log_error() {
    echo "[ERROR] $1" >&2
}

# Environment validation
validate_environment() {
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        log_error "Docker is not running"
        return 1
    fi

    # Check if Docker Compose is available
    if ! command -v docker-compose >/dev/null 2>&1; then
        log_error "Docker Compose is not installed"
        return 1
    fi

    return 0
}

# Service URL configuration
SERVICE_URL="http://localhost:${PORT:-8080}"
SERVICE_HEALTH_URL="${SERVICE_URL}/health" 