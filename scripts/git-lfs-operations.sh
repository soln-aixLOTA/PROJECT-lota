#!/bin/bash

# Git LFS Operations Script
# Handles rebasing, committing, fetching, and large file management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
echo -e "${BLUE}Git LFS Operations Tool${NC}"
echo "=========================="

# Function to check Git LFS status
check_lfs() {
    if ! command -v git-lfs &> /dev/null; then
        echo -e "${RED}Error: git-lfs is not installed${NC}"
        exit 1
    fi

    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}Error: not in a git repository${NC}"
        exit 1
    fi
}

# Function to fetch LFS objects
fetch_lfs() {
    echo -e "\n${GREEN}Fetching LFS objects...${NC}"
    git lfs fetch --all
    git lfs pull
    echo -e "${GREEN}LFS objects fetched successfully${NC}"
}

# Function to commit with LFS
commit_lfs() {
    local message="$1"
    if [ -z "$message" ]; then
        echo -e "${YELLOW}Please enter a commit message:${NC}"
        read -r message
    fi

    echo -e "\n${GREEN}Checking LFS files status...${NC}"
    git lfs status

    echo -e "\n${YELLOW}The following files will be committed:${NC}"
    git status --short

    echo -e "\n${YELLOW}Proceed with commit? (y/N)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        git add .
        git commit -m "$message"
        echo -e "${GREEN}Changes committed successfully${NC}"
    else
        echo -e "${YELLOW}Commit cancelled${NC}"
    fi
}

# Function to rebase with LFS
rebase_lfs() {
    local target_branch="$1"
    if [ -z "$target_branch" ]; then
        echo -e "${YELLOW}Please enter the target branch (e.g., main):${NC}"
        read -r target_branch
    fi

    echo -e "\n${GREEN}Preparing to rebase onto $target_branch...${NC}"
    
    # Fetch latest changes and LFS objects
    echo -e "${BLUE}Fetching latest changes...${NC}"
    git fetch --all
    git lfs fetch --all
    
    # Store current branch name
    current_branch=$(git symbolic-ref --short HEAD)
    
    echo -e "\n${YELLOW}Would you like to proceed with rebase onto $target_branch? (y/N)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        # Attempt rebase
        if git rebase "$target_branch"; then
            echo -e "${GREEN}Rebase completed successfully${NC}"
            
            # Ensure LFS objects are up to date
            git lfs pull
            
            echo -e "${GREEN}LFS objects updated${NC}"
        else
            echo -e "${RED}Rebase encountered conflicts${NC}"
            echo -e "${YELLOW}Please resolve conflicts and then:${NC}"
            echo "1. Fix the conflicts"
            echo "2. git add <resolved-files>"
            echo "3. git rebase --continue"
            echo "4. git lfs pull"
            exit 1
        fi
    else
        echo -e "${YELLOW}Rebase cancelled${NC}"
    fi
}

# Function to clean up large files
cleanup_large_files() {
    echo -e "\n${BLUE}Large File Cleanup${NC}"
    echo "===================="

    # Check for large files
    echo -e "\n${GREEN}Checking for large files (>100MB)...${NC}"
    large_files=$(find . -type f -size +100M ! -path "./.git/*" ! -path "*/.terraform/*")

    if [ -n "$large_files" ]; then
        echo -e "${YELLOW}Found large files:${NC}"
        echo "$large_files"
        
        echo -e "\n${YELLOW}Would you like to:"
        echo "1. Move files to Git LFS"
        echo "2. Add to .gitignore"
        echo "3. Remove files from repository"
        echo "4. Cancel"
        echo -e "Choose an option (1-4):${NC}"
        read -r option

        case "$option" in
            1)
                echo -e "\n${GREEN}Moving files to Git LFS...${NC}"
                for file in $large_files; do
                    # Get file extension
                    ext="${file##*.}"
                    # Add to Git LFS
                    git lfs track "*.$ext"
                    git add .gitattributes
                    git add "$file"
                done
                git commit -m "Move large files to Git LFS"
                ;;
            2)
                echo -e "\n${GREEN}Adding to .gitignore...${NC}"
                for file in $large_files; do
                    echo "$file" >> .gitignore
                done
                git add .gitignore
                git commit -m "Add large files to .gitignore"
                ;;
            3)
                echo -e "\n${GREEN}Removing files from repository...${NC}"
                for file in $large_files; do
                    git rm --cached "$file"
                done
                git commit -m "Remove large files from repository"
                ;;
            4)
                echo -e "${YELLOW}Cleanup cancelled${NC}"
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                ;;
        esac
    else
        echo -e "${GREEN}No large files found${NC}"
    fi
}

# Function to configure Terraform
configure_terraform() {
    echo -e "\n${BLUE}Terraform Configuration${NC}"
    echo "======================"

    # Add Terraform-specific entries to .gitignore
    echo -e "\n${GREEN}Adding Terraform entries to .gitignore...${NC}"
    cat << EOF >> .gitignore

# Terraform
.terraform/
*.tfstate
*.tfstate.*
crash.log
crash.*.log
*.tfvars
*.tfvars.json
override.tf
override.tf.json
*_override.tf
*_override.tf.json
.terraformrc
terraform.rc
EOF

    # Add Terraform provider patterns to Git LFS
    echo -e "\n${GREEN}Configuring Git LFS for Terraform providers...${NC}"
    git lfs track "*.terraform-provider-*"
    
    # Remove any existing .terraform directories from Git
    if [ -d ".terraform" ]; then
        echo -e "\n${YELLOW}Found .terraform directory. Remove from Git? (y/N)${NC}"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            git rm -rf --cached .terraform/
            rm -rf .terraform/
            echo -e "${GREEN}Removed .terraform directory${NC}"
        fi
    fi

    # Commit changes
    echo -e "\n${YELLOW}Commit these changes? (y/N)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        git add .gitignore .gitattributes
        git commit -m "Configure Terraform files and Git LFS"
        echo -e "${GREEN}Changes committed successfully${NC}"
    else
        echo -e "${YELLOW}Changes not committed${NC}"
    fi
}

# Main menu
show_menu() {
    echo -e "\n${BLUE}Available Operations:${NC}"
    echo "1. Fetch LFS objects"
    echo "2. Commit changes"
    echo "3. Rebase branch"
    echo "4. Clean up large files"
    echo "5. Configure Terraform"
    echo "6. Exit"
    echo -e "${YELLOW}Choose an operation (1-6):${NC}"
}

# Check Git LFS installation
check_lfs

# Process command line arguments
if [ $# -gt 0 ]; then
    case "$1" in
        "fetch")
            fetch_lfs
            ;;
        "commit")
            commit_lfs "$2"
            ;;
        "rebase")
            rebase_lfs "$2"
            ;;
        "cleanup")
            cleanup_large_files
            ;;
        "terraform")
            configure_terraform
            ;;
        *)
            echo -e "${RED}Unknown operation: $1${NC}"
            echo "Usage: $0 [fetch|commit|rebase|cleanup|terraform] [args]"
            exit 1
            ;;
    esac
    exit 0
fi

# Interactive menu
while true; do
    show_menu
    read -r choice
    case "$choice" in
        1)
            fetch_lfs
            ;;
        2)
            commit_lfs
            ;;
        3)
            rebase_lfs
            ;;
        4)
            cleanup_large_files
            ;;
        5)
            configure_terraform
            ;;
        6)
            echo -e "${GREEN}Goodbye!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            ;;
    esac
done 