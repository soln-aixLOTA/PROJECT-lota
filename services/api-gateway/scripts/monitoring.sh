#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Get the script's directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Get the project root directory (two levels up from script directory)
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../../../" && pwd )"

# Function to check if docker is running
check_docker() {
    if ! docker info >/dev/null 2>&1; then
        echo -e "${RED}Error: Docker is not running${NC}"
        exit 1
    fi
}

# Function to start the monitoring stack
start_monitoring() {
    echo -e "${YELLOW}Starting monitoring stack from ${PROJECT_ROOT}${NC}"
    cd "${PROJECT_ROOT}" || exit 1
    docker-compose up -d prometheus grafana
    
    echo -e "${YELLOW}Waiting for services to be ready...${NC}"
    sleep 10
    
    # Check if services are running
    if docker-compose ps | grep -q "prometheus.*Up"; then
        echo -e "${GREEN}Prometheus is running${NC}"
    else
        echo -e "${RED}Error: Prometheus failed to start${NC}"
    fi
    
    if docker-compose ps | grep -q "grafana.*Up"; then
        echo -e "${GREEN}Grafana is running${NC}"
        echo -e "${GREEN}Grafana UI is available at http://localhost:3000${NC}"
        echo -e "${YELLOW}Default credentials: admin/admin${NC}"
    else
        echo -e "${RED}Error: Grafana failed to start${NC}"
    fi
}

# Function to stop the monitoring stack
stop_monitoring() {
    echo -e "${YELLOW}Stopping monitoring stack...${NC}"
    cd "${PROJECT_ROOT}" || exit 1
    docker-compose stop prometheus grafana
    echo -e "${GREEN}Monitoring stack stopped${NC}"
}

# Function to restart the monitoring stack
restart_monitoring() {
    stop_monitoring
    start_monitoring
}

# Function to check monitoring stack status
status_monitoring() {
    echo -e "${YELLOW}Checking monitoring stack status...${NC}"
    cd "${PROJECT_ROOT}" || exit 1
    
    if docker-compose ps | grep -q "prometheus.*Up"; then
        echo -e "${GREEN}Prometheus is running${NC}"
    else
        echo -e "${RED}Prometheus is not running${NC}"
    fi
    
    if docker-compose ps | grep -q "grafana.*Up"; then
        echo -e "${GREEN}Grafana is running${NC}"
    else
        echo -e "${RED}Grafana is not running${NC}"
    fi
}

# Function to view logs
view_logs() {
    cd "${PROJECT_ROOT}" || exit 1
    if [ "$1" == "prometheus" ] || [ "$1" == "grafana" ]; then
        echo -e "${YELLOW}Showing logs for $1...${NC}"
        docker-compose logs --tail=100 -f "$1"
    else
        echo -e "${RED}Error: Please specify 'prometheus' or 'grafana'${NC}"
        exit 1
    fi
}

# Function to reload Prometheus configuration
reload_prometheus() {
    echo -e "${YELLOW}Reloading Prometheus configuration...${NC}"
    curl -X POST http://localhost:9090/-/reload
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Prometheus configuration reloaded successfully${NC}"
    else
        echo -e "${RED}Error: Failed to reload Prometheus configuration${NC}"
    fi
}

# Main script
check_docker

case "$1" in
    start)
        start_monitoring
        ;;
    stop)
        stop_monitoring
        ;;
    restart)
        restart_monitoring
        ;;
    status)
        status_monitoring
        ;;
    logs)
        view_logs "$2"
        ;;
    reload)
        reload_prometheus
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|reload}"
        echo "  start   - Start the monitoring stack"
        echo "  stop    - Stop the monitoring stack"
        echo "  restart - Restart the monitoring stack"
        echo "  status  - Check monitoring stack status"
        echo "  logs    - View logs (prometheus|grafana)"
        echo "  reload  - Reload Prometheus configuration"
        exit 1
        ;;
esac

exit 0 