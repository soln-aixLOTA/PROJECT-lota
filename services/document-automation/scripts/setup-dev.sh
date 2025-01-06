#!/bin/bash

# Script to set up development environment for Document Automation Service

set -e  # Exit on error

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed"
        log_info "Install Rust using: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        log_info "Install Docker from: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        log_info "Install Docker Compose from: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    log_info "All prerequisites satisfied"
}

# Create development configuration
setup_config() {
    log_info "Setting up configuration..."
    
    if [ ! -f "config/default.example.toml" ]; then
        log_error "Example configuration file not found"
        exit 1
    fi
    
    if [ -f "config/default.toml" ]; then
        log_warn "Configuration file already exists, skipping..."
    else
        cp config/default.example.toml config/default.toml
        log_info "Created default configuration"
    fi
}

# Set up development database
setup_database() {
    log_info "Setting up database..."
    
    # Start PostgreSQL container
    docker compose up -d db
    
    # Wait for database to be ready
    log_info "Waiting for database to be ready..."
    for i in {1..30}; do
        if docker compose exec db pg_isready -U postgres &> /dev/null; then
            break
        fi
        sleep 1
        if [ "$i" -eq 30 ]; then
            log_error "Database failed to start"
            exit 1
        fi
    done
    
    # Run migrations
    log_info "Running database migrations..."
    cargo sqlx migrate run
}

# Set up storage directory
setup_storage() {
    log_info "Setting up storage directory..."
    
    mkdir -p data/documents
    chmod 755 data/documents
    log_info "Created storage directory: data/documents"
}

# Install development tools
install_tools() {
    log_info "Installing development tools..."
    
    # Install jwt-cli for token management
    if ! command -v jwt &> /dev/null; then
        cargo install jwt-cli
        log_info "Installed jwt-cli"
    fi
    
    # Install openapi-generator for client generation
    if ! command -v openapi-generator &> /dev/null; then
        cargo install openapi-generator-cli
        log_info "Installed openapi-generator"
    fi
}

# Generate development token
generate_token() {
    log_info "Generating development token..."
    
    if [ ! -f "scripts/dev-token.sh" ]; then
        log_error "Development token script not found"
        exit 1
    fi
    
    TOKEN=$(./scripts/dev-token.sh)
    echo "export AUTH_TOKEN=\"$TOKEN\"" > .env.dev
    log_info "Generated token and saved to .env.dev"
}

# Build the project
build_project() {
    log_info "Building project..."
    
    cargo build
    log_info "Project built successfully"
}

# Main setup process
main() {
    log_info "Starting development environment setup..."
    
    check_prerequisites
    setup_config
    setup_database
    setup_storage
    install_tools
    generate_token
    build_project
    
    log_info "Development environment setup complete!"
    log_info "To start the service, run: cargo run"
    log_info "To load environment variables, run: source .env.dev"
}

# Run main function
main 