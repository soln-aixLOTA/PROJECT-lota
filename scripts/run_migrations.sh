#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
    source .env
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    exit 1
fi

# Run migrations
echo "Running migrations..."
cargo sqlx migrate run

echo "Migrations completed successfully" 