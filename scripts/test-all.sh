#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'  # No Color

# Test configuration
TEST_TOKEN="TEST_TOKEN_DO_NOT_USE_IN_PRODUCTION"

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

# Test scenarios
test_scenarios() {
    local scenarios=("crud" "workflow" "mixed" "security")
    local failed=0

    for scenario in "${scenarios[@]}"; do
        log_info "Running $scenario scenario..."
        if ! ./load-test.sh --scenario "$scenario" --token "$TEST_TOKEN" --test; then
            log_error "$scenario scenario failed"
            failed=$((failed + 1))
        else
            log_info "$scenario scenario completed successfully"
        fi
        echo
        sleep 2  # Brief pause between scenarios
    done

    # Test with metrics output
    log_info "Running mixed scenario with Prometheus metrics..."
    if ! ./load-test.sh --scenario mixed --output prometheus --token "$TEST_TOKEN" --test; then
        log_error "Mixed scenario with Prometheus metrics failed"
        failed=$((failed + 1))
    else
        log_info "Mixed scenario with Prometheus metrics completed successfully"
    fi

    return $failed
}

# Main function
main() {
    # Make load-test.sh executable if it isn't already
    chmod +x load-test.sh

    log_info "Starting test suite..."

    # Validate environment
    if ! ./load-test.sh --help &> /dev/null; then
        log_error "load-test.sh script not found or not executable"
        exit 1
    fi

    # Run all test scenarios
    if ! test_scenarios; then
        log_error "Some tests failed"
        exit 1
    fi

    log_info "All tests completed successfully"
}

# Run main function
main
