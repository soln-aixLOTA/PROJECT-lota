#!/bin/bash

# Default environment
ENV="dev"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        start|stop|restart|status)
            COMMAND="$1"
            shift
            ;;
        --env)
            ENV="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate environment
if [[ ! -f "config/${ENV}/config.sh" ]]; then
    echo "Error: Environment '${ENV}' not found"
    exit 1
fi

# Source environment-specific configuration
source "config/${ENV}/config.sh"

# Function to check if Docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        echo "Error: Docker is not running"
        exit 1
    fi
}

# Function to check server status
check_status() {
    local health_url="http://localhost:${PORT:-8080}/health"
    if curl -s -f "$health_url" >/dev/null 2>&1; then
        echo "Server is running"
        return 0
    else
        echo "Server is not running"
        return 1
    fi
}

# Function to start the server
start_server() {
    echo "Starting Document Automation Service..."
    docker compose up -d
    sleep 5
    check_status
}

# Function to stop the server
stop_server() {
    echo "Stopping Document Automation Service..."
    docker compose down
}

# Function to restart the server
restart_server() {
    echo "Restarting Document Automation Service..."
    docker compose restart
    sleep 5
    check_status
}

# Main logic
check_docker

case $COMMAND in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        restart_server
        ;;
    status)
        check_status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status} [--env {dev|prod}]"
        exit 1
        ;;
esac 