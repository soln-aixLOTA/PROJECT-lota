#!/bin/bash

# Update Dependencies Script
# Handles updating dependencies and fixing security vulnerabilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}Dependency Update Tool${NC}"
echo "=========================="

# Function to update Cargo.toml dependencies
update_cargo_deps() {
    local cargo_toml="$1"
    echo -e "\n${GREEN}Updating dependencies in $cargo_toml${NC}"
    
    # Update specific dependencies with security fixes
    sed -i 's/idna = "0.4.0"/idna = "1.0.0"/' "$cargo_toml"
    sed -i 's/sqlx = { version = "0.7.4"/sqlx = { version = "0.8.1"/' "$cargo_toml"
    sed -i 's/opentelemetry = "0.20.0"/opentelemetry = "0.21.0"/' "$cargo_toml"
    sed -i 's/opentelemetry-otlp = "0.13.0"/opentelemetry-otlp = "0.14.0"/' "$cargo_toml"
    
    # Update other dependencies
    cargo update --manifest-path "$cargo_toml"
}

# Function to check security after updates
check_security() {
    echo -e "\n${GREEN}Checking security vulnerabilities...${NC}"
    cargo audit
}

# Main update process
echo -e "\n${YELLOW}Starting dependency updates...${NC}"

# Update dependencies in each service
for service in services/*/; do
    if [ -f "${service}Cargo.toml" ]; then
        echo -e "\n${BLUE}Updating ${service%/}...${NC}"
        update_cargo_deps "${service}Cargo.toml"
    fi
done

# Update root dependencies if they exist
if [ -f "Cargo.toml" ]; then
    update_cargo_deps "Cargo.toml"
fi

# Run tests to ensure updates haven't broken anything
echo -e "\n${YELLOW}Running tests...${NC}"
cargo test

# Check security vulnerabilities
check_security

echo -e "\n${GREEN}Dependency updates completed!${NC}"
echo -e "${YELLOW}Please review the changes and commit them.${NC}" 