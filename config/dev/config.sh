#!/bin/bash

# Server configuration
export PORT=8080
export HOST="0.0.0.0"
export WORKERS=4

# Database configuration
export DB_USER="dev_user"
export DB_PASSWORD="dev_password"
export DB_NAME="document_automation_dev"
export DB_PORT=5432
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

# Security configuration
export JWT_SECRET="development_secret_key"
export TOKEN_EXPIRATION_HOURS=24

# Storage configuration
export STORAGE_PROVIDER="local"
export STORAGE_BUCKET="documents"
export STORAGE_REGION="us-east-1"

# Monitoring configuration
export GRAFANA_PASSWORD="admin"
export PROMETHEUS_PORT=9090
export GRAFANA_PORT=3000

# Logging configuration
export RUST_LOG="info" 