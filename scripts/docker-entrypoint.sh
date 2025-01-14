#!/bin/bash
set -e

# Start Docker daemon in the background
if [ -x "$(command -v dockerd)" ]; then
    dockerd &
    # Wait for Docker daemon to be ready
    while ! docker info >/dev/null 2>&1; do
        echo "Waiting for Docker daemon to be ready..."
        sleep 1
    done
    echo "Docker daemon is ready"
fi

# Execute the command passed to the script
exec "$@"
