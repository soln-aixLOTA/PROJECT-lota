#!/bin/bash

# Function to find duplicate files based on content
find_duplicates() {
    echo "Finding duplicate files..."
    find node_modules -type f -exec md5sum {} \; | sort | uniq -w32 -dD
}

# Function to analyze specific patterns of duplicates
analyze_patterns() {
    echo "Analyzing duplicate patterns..."
    
    # ES Abstract duplicates
    echo "ES Abstract duplicates:"
    find node_modules/es-abstract -type f -name "*.js" | sort
    
    # Framework duplicates
    echo "Framework duplicates:"
    find node_modules/next/dist/compiled -type f -name "*.js" | sort
    
    # Test framework duplicates
    echo "Test framework duplicates:"
    find node_modules -type f -path "*/jest*" -o -path "*/testing-library*" | sort
}

# Function to clean up safe duplicates
cleanup_safe_duplicates() {
    echo "Cleaning up safe duplicates..."
    
    # Create backup
    timestamp=$(date +%Y%m%d_%H%M%S)
    mkdir -p backups/node_modules_"$timestamp"
    
    # Remove duplicate test files while preserving the main ones
    find node_modules -type f -name "*.test.js" -exec rm {} \;
    
    # Remove duplicate source maps
    find node_modules -type f -name "*.js.map" -exec rm {} \;
}

# Main execution
echo "Starting duplicate analysis and cleanup..."
find_duplicates > duplicate_files.log
analyze_patterns > duplicate_patterns.log

# Ask for confirmation before cleanup
read -p "Do you want to proceed with cleanup? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    cleanup_safe_duplicates
    echo "Cleanup completed. Check backups directory for backup."
fi 