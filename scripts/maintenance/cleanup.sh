#!/bin/bash

# Cleanup script for maintenance tasks

# Remove Python cache files
find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null
find . -name "*.pyc" -delete
find . -name "*.pyo" -delete
find . -name "*.pyd" -delete

# Remove temporary files
find . -name "*.tmp" -delete
find . -name "*.temp" -delete
find . -name ".DS_Store" -delete

# Remove build artifacts
find . -name "*.o" -delete
find . -name "*.so" -delete

# Clean Rust build artifacts
if [ -d "target" ]; then
    cargo clean
fi

echo "Cleanup completed successfully!" 