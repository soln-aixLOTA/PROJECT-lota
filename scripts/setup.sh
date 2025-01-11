#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "LotaBots Platform - Prerequisites Setup"
echo "======================================"

# Check if script is run with sudo
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Please run as root or with sudo${NC}"
    exit 1
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check version
version_greater_equal() {
    printf '%s\n%s\n' "$2" "$1" | sort -V -C
}

# Check/Install Rust
check_rust() {
    if ! command_exists rustc; then
        echo -e "${YELLOW}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME"/.cargo/env
    fi
    
    RUST_VERSION=$(rustc --version | cut -d ' ' -f 2)
    if version_greater_equal "$RUST_VERSION" "1.55.0"; then
        echo -e "${GREEN}✓ Rust $RUST_VERSION installed${NC}"
    else
        echo -e "${RED}✗ Rust version must be 1.55.0 or higher (current: $RUST_VERSION)${NC}"
        exit 1
    fi
}

# Check/Install Docker
check_docker() {
    if ! command_exists docker; then
        echo -e "${YELLOW}Installing Docker...${NC}"
        curl -fsSL https://get.docker.com -o get-docker.sh
        sh get-docker.sh
        rm get-docker.sh
        systemctl enable docker
        systemctl start docker
    fi
    
    DOCKER_VERSION=$(docker --version | cut -d ' ' -f 3 | cut -d ',' -f 1)
    if version_greater_equal "$DOCKER_VERSION" "20.10.0"; then
        echo -e "${GREEN}✓ Docker $DOCKER_VERSION installed${NC}"
    else
        echo -e "${RED}✗ Docker version must be 20.10.0 or higher (current: $DOCKER_VERSION)${NC}"
        exit 1
    fi
}

# Check/Install kubectl
check_kubectl() {
    if ! command_exists kubectl; then
        echo -e "${YELLOW}Installing kubectl...${NC}"
        curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
        chmod +x kubectl
        mv kubectl /usr/local/bin/
    fi
    
    KUBECTL_VERSION=$(kubectl version --client -o json | grep -o '"gitVersion": *"[^"]*"' | cut -d'"' -f4 | cut -d'v' -f2)
    if version_greater_equal "$KUBECTL_VERSION" "1.21.0"; then
        echo -e "${GREEN}✓ kubectl $KUBECTL_VERSION installed${NC}"
    else
        echo -e "${RED}✗ kubectl version must be 1.21.0 or higher (current: $KUBECTL_VERSION)${NC}"
        exit 1
    fi
}

# Check/Install PostgreSQL client
check_postgres_client() {
    if ! command_exists psql; then
        echo -e "${YELLOW}Installing PostgreSQL client...${NC}"
        apt-get update
        apt-get install -y postgresql-client
    fi
    echo -e "${GREEN}✓ PostgreSQL client installed${NC}"
}

# Check/Install Redis client
check_redis_client() {
    if ! command_exists redis-cli; then
        echo -e "${YELLOW}Installing Redis client...${NC}"
        apt-get update
        apt-get install -y redis-tools
    fi
    echo -e "${GREEN}✓ Redis client installed${NC}"
}

# Main installation process
main() {
    echo "Checking system prerequisites..."
    
    # Update package list
    apt-get update
    
    # Install basic requirements
    apt-get install -y \
        curl \
        build-essential \
        pkg-config \
        libssl-dev \
        git
    
    # Check all components
    check_rust
    check_docker
    check_kubectl
    check_postgres_client
    check_redis_client
    
    echo -e "\n${GREEN}All prerequisites have been installed successfully!${NC}"
    echo "Please ensure you have:"
    echo "1. Set up your Kubernetes cluster"
    echo "2. Configured Vault (VAULT_ADDR, VAULT_ROLE, VAULT_NAMESPACE)"
    echo "3. Set up PostgreSQL and Redis servers"
}

main 