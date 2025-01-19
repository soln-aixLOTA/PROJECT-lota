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
# WARNING: Set these securely in your production environment
# Use a secure secret management solution like AWS Secrets Manager or HashiCorp Vault
export DB_PASSWORD=""

# JWT configuration
# WARNING: Set these securely in your production environment
# Use a secure secret management solution like AWS Secrets Manager or HashiCorp Vault
export JWT_SECRET="${JWT_SECRET}"  # Must be set in environment
export JWT_EXPIRATION=3600  # 1 hour

# Test configuration
export TEST_MODE="false"
export TEST_VUS=50  # Virtual users for load testing
export TEST_DURATION="5m"

# Monitoring configuration
export PROMETHEUS_URL="https://prometheus.monitoring.example.com"
export INFLUXDB_URL="https://influxdb.monitoring.example.com"
