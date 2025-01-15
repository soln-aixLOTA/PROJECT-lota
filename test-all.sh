#!/bin/bash

# Source common functions
source ./lib/common.sh

# Set environment to test by default
source ./set-env.sh --env test

log_info "Starting test suite..."

# Generate test token
TEST_TOKEN=$(./dev-token.sh --user test --role admin --key "$JWT_SECRET" --expiration "$JWT_EXPIRATION")
if [ -z "$TEST_TOKEN" ]; then
    log_error "Failed to generate test token"
    exit 1
fi

log_info "Generated token: $TEST_TOKEN"

# Run CRUD scenario
log_info "Running crud scenario..."
./load-test.sh --url "$SERVICE_URL" --scenario crud --token "$TEST_TOKEN" --test
if [ $? -eq 0 ]; then
    log_info "crud scenario completed successfully"
else
    log_error "crud scenario failed"
    exit 1
fi

# Run Workflow scenario
log_info "Running workflow scenario..."
./load-test.sh --url "$SERVICE_URL" --scenario workflow --token "$TEST_TOKEN" --test
if [ $? -eq 0 ]; then
    log_info "workflow scenario completed successfully"
else
    log_error "workflow scenario failed"
    exit 1
fi

# Run Mixed scenario
log_info "Running mixed scenario..."
./load-test.sh --url "$SERVICE_URL" --scenario mixed --token "$TEST_TOKEN" --test
if [ $? -eq 0 ]; then
    log_info "mixed scenario completed successfully"
else
    log_error "mixed scenario failed"
    exit 1
fi

# Run Security scenario
log_info "Running security scenario..."
./load-test.sh --url "$SERVICE_URL" --scenario security --token "$TEST_TOKEN" --test
if [ $? -eq 0 ]; then
    log_info "security scenario completed successfully"
else
    log_error "security scenario failed"
    exit 1
fi

# Run Mixed scenario with Prometheus metrics
log_info "Running mixed scenario with Prometheus metrics..."
./load-test.sh --url "$SERVICE_URL" --scenario mixed --token "$TEST_TOKEN" --output prometheus --test
if [ $? -eq 0 ]; then
    log_info "Mixed scenario with Prometheus metrics completed successfully"
else
    log_error "Mixed scenario with Prometheus metrics failed"
    exit 1
fi

log_info "All tests completed successfully"
