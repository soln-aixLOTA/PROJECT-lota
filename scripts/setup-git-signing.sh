#!/usr/bin/env bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() { echo -e "${BLUE}INFO:${NC} $1"; }
print_success() { echo -e "${GREEN}SUCCESS:${NC} $1"; }
print_warning() { echo -e "${YELLOW}WARNING:${NC} $1"; }
print_error() { echo -e "${RED}ERROR:${NC} $1"; }

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to setup GPG signing
setup_gpg() {
    print_info "Setting up GPG signing..."

    if ! command_exists gpg; then
        print_error "GPG is not installed. Please install it first."
        exit 1
    }

    # Check for existing GPG keys
    if gpg --list-secret-keys --keyid-format=long | grep -q "sec"; then
        print_warning "Existing GPG keys found. Do you want to:"
        echo "1) Use an existing key"
        echo "2) Generate a new key"
        read -p "Choose (1/2): " choice

        if [ "$choice" = "1" ]; then
            gpg --list-secret-keys --keyid-format=long
            read -p "Enter the key ID to use (e.g., 3AA5C34371567BD2): " key_id
        else
            # Generate new key
            print_info "Generating new GPG key..."
            gpg --full-generate-key
            key_id=$(gpg --list-secret-keys --keyid-format=long | grep sec | head -n 1 | awk '{print $2}' | cut -d'/' -f2)
        fi
    else
        # No existing keys, generate new one
        print_info "No existing GPG keys found. Generating new key..."
        gpg --full-generate-key
        key_id=$(gpg --list-secret-keys --keyid-format=long | grep sec | head -n 1 | awk '{print $2}' | cut -d'/' -f2)
    fi

    # Configure Git to use GPG
    git config --global user.signingkey "$key_id"
    git config --global commit.gpgsign true

    # Export public key for GitHub
    print_info "Your public GPG key (add this to GitHub):"
    gpg --armor --export "$key_id"

    print_success "GPG signing setup complete!"
    print_info "Add the above public key to GitHub at: https://github.com/settings/keys"
}

# Function to setup SSH signing
setup_ssh() {
    print_info "Setting up SSH signing..."

    if ! command_exists ssh-keygen; then
        print_error "ssh-keygen is not installed. Please install OpenSSH first."
        exit 1
    }

    # Check for existing SSH keys
    if [ -f ~/.ssh/id_ed25519 ]; then
        print_warning "Existing SSH key found. Do you want to:"
        echo "1) Use existing key"
        echo "2) Generate new key"
        read -p "Choose (1/2): " choice

        if [ "$choice" = "2" ]; then
            read -p "Enter your email: " email
            ssh-keygen -t ed25519 -C "$email"
        fi
    else
        read -p "Enter your email: " email
        ssh-keygen -t ed25519 -C "$email"
    fi

    # Configure Git to use SSH signing
    git config --global gpg.format ssh
    git config --global user.signingkey "$(realpath ~/.ssh/id_ed25519.pub)"

    # Display public key
    print_info "Your public SSH key (add this to GitHub):"
    cat ~/.ssh/id_ed25519.pub

    print_success "SSH signing setup complete!"
    print_info "Add the above public key to GitHub at: https://github.com/settings/keys"
}

# Function to setup S/MIME signing
setup_smime() {
    print_info "Setting up S/MIME signing..."

    if ! command_exists smimesign; then
        print_error "smimesign is not installed. Please install it first."
        exit 1
    }

    print_info "Please ensure you have your X.509 certificate ready."
    read -p "Enter the path to your certificate file: " cert_path

    if [ ! -f "$cert_path" ]; then
        print_error "Certificate file not found!"
        exit 1
    }

    # Configure Git to use S/MIME
    git config --global gpg.x509.program smimesign
    git config --global gpg.format x509

    print_success "S/MIME signing setup complete!"
}

# Main menu
echo "LotaBots Git Signing Setup"
echo "========================="
echo "Choose signing method:"
echo "1) GPG (Recommended)"
echo "2) SSH (Alternative)"
echo "3) S/MIME (For organizations)"
echo "4) Exit"

read -p "Select option (1-4): " option

case $option in
    1)
        setup_gpg
        ;;
    2)
        setup_ssh
        ;;
    3)
        setup_smime
        ;;
    4)
        print_info "Exiting..."
        exit 0
        ;;
    *)
        print_error "Invalid option"
        exit 1
        ;;
esac

# Final instructions
print_info "To verify your setup:"
echo "1. Try creating a signed commit:"
echo "   git commit -S -m \"test: verify commit signing\""
echo "2. Verify the signature:"
echo "   git verify-commit HEAD"
echo "3. Push to GitHub and check for the 'Verified' badge"

print_success "Setup complete! Happy coding!"
