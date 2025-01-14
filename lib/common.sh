#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'  # No Color

# Log functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Make scripts executable
make_executable() {
    local script="$1"
    if [ ! -x "$script" ]; then
        chmod +x "$script"
    fi
}

# Validate script exists
validate_script() {
    local script="$1"
    if [ ! -f "$script" ]; then
        log_error "$script not found"
        return 1
    fi
    return 0
}

# Environment validation
validate_environment() {
    log_info "Validating environment..."
    
    # Check Docker installation
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        log_info "Please install Docker first: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        log_info "Please start Docker daemon first"
        exit 1
    fi
    
    # Check if service is running (optional in test mode)
    if [ -z "$TEST_MODE" ]; then
        if ! curl -s "http://localhost:8080/health" &> /dev/null; then
            log_warn "Document Automation Service is not running"
            log_info "Some tests may fail if the service is not available"
        fi
        
        # Check database connection (optional in test mode)
        if [ -f "./db-manage.sh" ]; then
            if ! ./db-manage.sh check &> /dev/null; then
                log_warn "Database connection failed"
                log_info "Some tests may fail if the database is not available"
            fi
        fi
    fi
    
    log_info "Environment validation completed"
}

# Check k6 installation
check_k6() {
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        log_info "Please install Docker first: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Pull k6 Docker image if not present
    if ! docker images | grep -q "grafana/k6"; then
        log_info "Pulling k6 Docker image..."
        if ! docker pull grafana/k6; then
            log_error "Failed to pull k6 Docker image"
            exit 1
        fi
    fi
    
    log_info "k6 Docker image is ready"
}

# Validate test results
validate_test_results() {
    local test_output="$1"
    local scenario="$2"
    
    # Skip validation in test mode
    if [ -n "$TEST_MODE" ]; then
        return 0
    fi
    
    log_info "Validating test results for scenario: $scenario"
    
    # Check for common error patterns
    if echo "$test_output" | grep -q "connection refused"; then
        log_error "Service connection failed during test"
        return 1
    fi
    
    if echo "$test_output" | grep -q "invalid token"; then
        log_warn "Authentication issues detected during test"
    fi
    
    # Validate scenario-specific metrics
    case "$scenario" in
        crud)
            if ! echo "$test_output" | grep -q "document_uploads"; then
                log_error "No document uploads recorded"
                return 1
            fi
            ;;
        workflow)
            if ! echo "$test_output" | grep -q "workflow_completions"; then
                log_error "No workflow completions recorded"
                return 1
            fi
            ;;
        security)
            if ! echo "$test_output" | grep -q "auth_successes"; then
                log_error "No successful authentication attempts recorded"
                return 1
            fi
            ;;
    esac
    
    return 0
} 