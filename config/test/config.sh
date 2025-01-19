#!/bin/bash

# Test environment configuration

# Service configuration
export SERVICE_URL="http://test-service:8080"
export SERVICE_HEALTH_URL="http://test-service:8080/health"

# Database configuration
export DB_HOST="test-db"
export DB_PORT="5432"
export DB_NAME="document_automation_test"
export DB_USER="test_user"
# WARNING: Set these securely in your CI/CD environment
export DB_PASSWORD=""

# JWT configuration
# WARNING: Set these securely in your CI/CD environment
# Generate with: openssl rand -hex 32
# Generate a random test-only JWT secret if not provided
if [ -z "$JWT_SECRET" ]; then
    # Use openssl to generate a secure random value prefixed with TEST_
    JWT_SECRET="TEST_$(openssl rand -hex 32)"
    log_warn "Using auto-generated JWT_SECRET for testing. This is OK for tests but not for production!"
fi
export JWT_SECRET
export JWT_EXPIRATION=3600  # 1 hour

# Test configuration
export TEST_MODE="true"
export TEST_VUS=5  # Virtual users for load testing
export TEST_DURATION="10s"

# Monitoring configuration
export PROMETHEUS_URL="http://test-prometheus:9090"
export INFLUXDB_URL="http://test-influxdb:8086"
