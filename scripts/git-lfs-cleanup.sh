#!/bin/bash

# Git LFS Cleanup Script
# This script helps manage and clean up Git LFS objects

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print header
echo -e "${GREEN}Git LFS Cleanup Tool${NC}"
echo "=========================="

# Function to print size in human readable format
human_size() {
    local size=$1
    local units=('B' 'KB' 'MB' 'GB' 'TB')
    local unit_index=0

    while ((size > 1024 && unit_index < 4)); do
        size=$(echo "scale=2; $size/1024" | bc)
        ((unit_index++))
    done

    echo "$size${units[$unit_index]}"
}

# Check if git-lfs is installed
if ! command -v git-lfs &> /dev/null; then
    echo -e "${RED}Error: git-lfs is not installed${NC}"
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: not in a git repository${NC}"
    exit 1
fi

# Show current LFS status
echo -e "\n${GREEN}Current LFS Status:${NC}"
git lfs status

# Calculate total size of LFS objects
echo -e "\n${GREEN}LFS Objects Size:${NC}"
total_size=$(git lfs ls-files -l | awk '{total += $4} END {print total}')
echo "Total size: $(human_size "$total_size")"

# List largest LFS objects
echo -e "\n${GREEN}Largest LFS Objects:${NC}"
git lfs ls-files -l | sort -nrk4 | head -n 10 | \
    awk '{printf "%-60s %s\n", $3, system("numfmt --to=iec-i --suffix=B " $4)}'

# Check for unused LFS objects
echo -e "\n${GREEN}Checking for unused LFS objects...${NC}"
git lfs prune --dry-run

# Prompt for cleanup
echo -e "\n${YELLOW}Would you like to perform cleanup? (y/N)${NC}"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo -e "\n${GREEN}Performing cleanup...${NC}"
    
    # Prune unused objects
    git lfs prune
    
    # Garbage collect
    git gc --prune=now
    
    # Verify LFS objects
    git lfs verify
    
    echo -e "\n${GREEN}Cleanup completed successfully!${NC}"
else
    echo -e "\n${YELLOW}Cleanup skipped.${NC}"
fi

# Show final status
echo -e "\n${GREEN}Final LFS Status:${NC}"
git lfs status

# Show maintenance recommendations
echo -e "\n${GREEN}Maintenance Recommendations:${NC}"
echo "1. Run this cleanup script regularly (e.g., monthly)"
echo "2. Use 'git lfs migrate' to optimize history if needed"
echo "3. Consider using 'git lfs fetch --recent' for shallow clones"
echo "4. Monitor LFS storage quotas on your Git hosting service" 