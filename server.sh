#!/bin/bash

# Source common functions
source ./lib/common.sh

# Default values
COMMAND="start"
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
        -h|--help)
            echo "Usage: $0 [command] [options]"
            echo
            echo "Commands:"
            echo "  start       Start the server (default)"
            echo "  stop        Stop the server"
            echo "  restart     Restart the server"
            echo "  status      Check server status"
            echo
            echo "Options:"
            echo "  --env ENV   Environment to use: dev|test|prod (default: dev)"
            echo "  -h, --help  Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Load environment configuration
source ./set-env.sh --env "$ENV"

# Docker compose file
COMPOSE_FILE="../docker-compose.yml"

# Check if Docker Compose file exists
if [ ! -f "$COMPOSE_FILE" ]; then
    log_error "Docker Compose file not found: $COMPOSE_FILE"
    exit 1
fi

# Function to check server status
check_status() {
    if curl -s "$SERVICE_HEALTH_URL" > /dev/null; then
        log_info "Server is running at $SERVICE_URL"
        return 0
    else
        log_warn "Server is not running"
        return 1
    fi
}

# Execute command
case "$COMMAND" in
    start)
        log_info "Starting Document Automation Service..."
        docker-compose -f "$COMPOSE_FILE" up -d
        sleep 5  # Wait for services to start
        check_status
        ;;
    stop)
        log_info "Stopping Document Automation Service..."
        docker-compose -f "$COMPOSE_FILE" down
        ;;
    restart)
        log_info "Restarting Document Automation Service..."
        docker-compose -f "$COMPOSE_FILE" restart
        sleep 5  # Wait for services to restart
        check_status
        ;;
    status)
        check_status
        ;;
esac 