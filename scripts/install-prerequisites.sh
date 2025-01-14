#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Installing prerequisites for LOTA AI...${NC}"

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to handle errors
handle_error() {
    echo -e "${RED}Error: $1${NC}"
    exit 1
}

# Clean up problematic repositories
echo -e "${BLUE}Cleaning up package sources...${NC}"
# Remove old Kubernetes repository
sudo rm -f /etc/apt/sources.list.d/kubernetes.list
# Remove old Google Cloud repository
sudo rm -f /etc/apt/sources.list.d/google-cloud-sdk.list

# Update package list
echo -e "${BLUE}Updating package list...${NC}"
sudo apt update || echo -e "${BLUE}Some repositories might be unavailable, continuing...${NC}"

# Ensure snap is installed
if ! command_exists snap; then
    echo -e "${BLUE}Installing snap...${NC}"
    sudo apt install snapd -y || handle_error "Failed to install snap"
    sudo snap wait system seed.loaded
fi

# Install GitHub CLI if not installed
if ! command_exists gh; then
    echo -e "${BLUE}Installing GitHub CLI...${NC}"
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg ||
        handle_error "Failed to download GitHub CLI key"
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list >/dev/null
    sudo apt update && sudo apt install gh -y || handle_error "Failed to install GitHub CLI"
fi

# Install Google Cloud SDK if not installed
if ! command_exists gcloud; then
    echo -e "${BLUE}Installing Google Cloud SDK...${NC}"

    # First try snap installation
    if sudo snap install google-cloud-cli --classic; then
        echo -e "${GREEN}Successfully installed Google Cloud SDK via snap${NC}"
        export GCLOUD_JUST_INSTALLED=1
    else
        echo -e "${BLUE}Snap installation failed, trying apt installation...${NC}"
        # Add the Cloud SDK distribution URI as a package source
        echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee /etc/apt/sources.list.d/google-cloud-sdk.list

        # Import the Google Cloud public key
        curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key --keyring /usr/share/keyrings/cloud.google.gpg add -

        # Update and install the Cloud SDK
        sudo apt update && sudo apt install google-cloud-cli -y || handle_error "Failed to install Google Cloud SDK"
    fi

    # Add gcloud to PATH if not already there
    if ! grep -q "google-cloud-sdk\|snap/bin" ~/.bashrc; then
        echo 'export PATH=$PATH:/snap/bin' >>~/.bashrc
        source ~/.bashrc
    fi
fi

# Install other required tools
echo -e "${BLUE}Installing other required tools...${NC}"
sudo apt install -y \
    curl \
    openssl \
    jq ||
    handle_error "Failed to install additional tools"

# Verify installations
echo -e "${BLUE}Verifying installations...${NC}"

echo -n "GitHub CLI: "
if command_exists gh; then
    echo -e "${GREEN}✓ Installed${NC} ($(gh --version | head -n 1))"
else
    echo -e "${RED}✗ Failed to install${NC}"
    exit 1
fi

echo -n "Google Cloud SDK: "
if command_exists gcloud; then
    echo -e "${GREEN}✓ Installed${NC} ($(gcloud --version | head -n 1))"
else
    echo -e "${RED}✗ Failed to install${NC}"
    echo -e "${BLUE}Try running: source ~/.bashrc${NC}"
    exit 1
fi

echo -e "\n${GREEN}✅ Prerequisites installation completed!${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo "1. Run this command to update your PATH:"
echo "   source ~/.bashrc"
echo "2. Run 'gh auth login' to authenticate with GitHub"
echo "3. Run 'gcloud auth login' to authenticate with Google Cloud"
echo "4. Run './scripts/setup-secrets.sh' to set up your secrets"

# Remind about shell reload if gcloud was just installed
if [[ -n "$GCLOUD_JUST_INSTALLED" ]]; then
    echo -e "\n${BLUE}Important:${NC} You need to reload your shell to use gcloud."
    echo "Run: source ~/.bashrc"
fi
