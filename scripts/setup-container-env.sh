#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to check file permissions
check_permissions() {
    local file=$1
    local perms=$(stat -c %a "$file")
    if [ "$perms" != "600" ]; then
        print_status "$YELLOW" "Warning: $file has incorrect permissions ($perms). Fixing..."
        chmod 600 "$file"
    fi
}

# Function to backup existing env file
backup_env() {
    if [ -f .env ]; then
        local backup_file=".env.backup.$(date +%Y%m%d_%H%M%S)"
        cp .env "$backup_file"
        chmod 600 "$backup_file"
        print_status "$BLUE" "Created backup of existing .env file: $backup_file"
    fi
}

# Function to safely update NGC API key
update_ngc_key() {
    local env_file=".env"
    local key=$1

    # Validate key format
    if [[ ! $key =~ ^nvapi-[a-zA-Z0-9_-]+$ ]]; then
        print_status "$RED" "Invalid NGC API key format. Key should start with 'nvapi-'"
        return 1
    fi

    # Check key length
    if [ ${#key} -lt 20 ]; then
        print_status "$RED" "NGC API key seems too short. Please verify the key."
        return 1
    fi

    # Update or add NGC_API_KEY in .env file
    if grep -q "^NGC_API_KEY=" "$env_file"; then
        sed -i "s|^NGC_API_KEY=.*|NGC_API_KEY=$key|" "$env_file"
    else
        echo "NGC_API_KEY=$key" >>"$env_file"
    fi

    print_status "$GREEN" "NGC API key has been securely updated"
}

# Function to verify Docker installation and permissions
check_docker() {
    if ! command -v docker &>/dev/null; then
        print_status "$RED" "Docker is not installed. Please install Docker first."
        exit 1
    fi

    if ! docker info &>/dev/null; then
        print_status "$RED" "Cannot connect to Docker daemon. Please check if Docker is running and you have proper permissions."
        exit 1
    fi
}

# Function to check for sensitive data in environment
check_sensitive_data() {
    local env_file=".env"
    if [ -f "$env_file" ]; then
        if grep -iE "(password|secret|key|token|credential)" "$env_file" | grep -vE "^#" | grep -q ":"; then
            print_status "$RED" "Warning: Possible plaintext credentials found in $env_file"
            print_status "$YELLOW" "Please ensure all sensitive data is properly encrypted or stored securely"
        fi
    fi
}

# Main setup process
echo "Container Update Environment Setup"
echo "================================"

# Check Docker installation
check_docker

# Create backup of existing .env file
backup_env

# Check if .env file exists
if [ ! -f .env ]; then
    print_status "$YELLOW" "No .env file found. Creating from template..."
    cp .env.example .env
    print_status "$GREEN" "Created .env file from template"
else
    print_status "$YELLOW" "Existing .env file found"
fi

# Check if NGC key is provided as argument
if [ -n "$1" ]; then
    update_ngc_key "$1"
else
    # Prompt for NGC API key
    echo
    print_status "$YELLOW" "Please enter your NGC API key (or press Enter to skip):"
    read -rs ngc_key # -s flag hides the input
    echo             # New line after hidden input

    if [ -n "$ngc_key" ]; then
        update_ngc_key "$ngc_key"
    else
        print_status "$YELLOW" "Skipping NGC API key setup"
    fi
fi

# Set proper permissions
chmod 600 .env
check_permissions .env

# Check for sensitive data
check_sensitive_data

# Test NGC authentication if key is present
if grep -q "^NGC_API_KEY=nvapi-" .env; then
    print_status "$BLUE" "Testing NGC authentication..."
    if ! docker login nvcr.io -u "\$oauthtoken" -p "$(grep '^NGC_API_KEY=' .env | cut -d= -f2)" &>/dev/null; then
        print_status "$RED" "NGC authentication failed. Please verify your API key."
    else
        print_status "$GREEN" "NGC authentication successful"
        # Cleanup Docker credentials
        docker logout nvcr.io &>/dev/null
    fi
fi

# Verify setup
echo
print_status "$GREEN" "Environment setup complete!"
echo
echo "Next steps:"
echo "1. Verify the contents of your .env file"
echo "2. Run the container update script: ./scripts/update_containers.sh"
echo "3. Consider setting up a secrets management solution for production"
echo
print_status "$YELLOW" "Security Reminders:"
echo "- Never commit .env files to version control"
echo "- Regularly rotate your NGC API key"
echo "- Keep backups of your .env file in a secure location"
echo "- Use different keys for development and production"
echo "- Monitor Docker security advisories regularly"
