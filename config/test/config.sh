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
export DB_PASSWORD="test_password"

# JWT configuration
export JWT_SECRET="test_secret_key"
export JWT_EXPIRATION=3600  # 1 hour

# Test configuration
export TEST_MODE="true"
export TEST_VUS=5  # Virtual users for load testing
export TEST_DURATION="10s"

# Monitoring configuration
export PROMETHEUS_URL="http://test-prometheus:9090"
export INFLUXDB_URL="http://test-influxdb:8086" 