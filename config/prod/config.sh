#!/bin/bash

# Production environment configuration

# Service configuration
export SERVICE_URL="https://api.document-automation.example.com"
export SERVICE_HEALTH_URL="https://api.document-automation.example.com/health"

# Database configuration
export DB_HOST="prod-db.internal"
export DB_PORT="5432"
export DB_NAME="document_automation_prod"
export DB_USER="prod_user"
export DB_PASSWORD="REPLACE_WITH_SECURE_PASSWORD"

# JWT configuration
export JWT_SECRET="REPLACE_WITH_SECURE_SECRET"
export JWT_EXPIRATION=3600  # 1 hour

# Test configuration
export TEST_MODE="false"
export TEST_VUS=50  # Virtual users for load testing
export TEST_DURATION="5m"

# Monitoring configuration
export PROMETHEUS_URL="https://prometheus.monitoring.example.com"
export INFLUXDB_URL="https://influxdb.monitoring.example.com" 