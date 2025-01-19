#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "LotaBots Platform - Setup"
echo "========================"

# Function to check if script exists and is executable
check_script() {
    local script=$1
    if [ ! -f "$script" ]; then
        echo -e "${RED}Error: $script not found${NC}"
        exit 1
    fi
    chmod +x "$script"
}

# Function to run a setup step
run_step() {
    local step=$1
    local script=$2
    
    echo -e "\n${YELLOW}Step $step: Running $script...${NC}"
    if ! ./"$script"; then
        echo -e "${RED}Failed to execute $script${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Completed $script${NC}"
}

# Main setup process
main() {
    # Ensure we're in the project root
    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}Error: Please run this script from the project root directory${NC}"
        exit 1
    fi
    
    # Check if all required scripts exist
    check_script "scripts/setup.sh"
    check_script "scripts/configure_env.sh"
    check_script "scripts/db_setup.sh"
    check_script "scripts/deploy.sh"
    
    # Run each setup step
    run_step 1 "scripts/setup.sh"
    run_step 2 "scripts/configure_env.sh"
    run_step 3 "scripts/db_setup.sh"
    
    # Ask if deployment should be performed
    read -p "Do you want to deploy to Kubernetes now? (y/N) " should_deploy
    if [[ $should_deploy =~ ^[Yy]$ ]]; then
        run_step 4 "scripts/deploy.sh"
    else
        echo -e "\n${YELLOW}Skipping deployment${NC}"
        echo "You can deploy later by running: ./scripts/deploy.sh"
    fi
    
    echo -e "\n${GREEN}Setup completed successfully!${NC}"
    echo "Next steps:"
    echo "1. Review the generated configuration in .env"
    echo "2. Start the application locally with 'cargo run' or deploy with './scripts/deploy.sh'"
    echo "3. Check the application logs and monitoring"
}

main 