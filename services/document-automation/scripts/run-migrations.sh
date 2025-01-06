#!/bin/bash
set -e

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    exit 1
fi

# Create migrations directory if it doesn't exist
MIGRATIONS_DIR="$(dirname "$0")/../migrations"
mkdir -p "$MIGRATIONS_DIR"

# Run migrations
for migration in "$MIGRATIONS_DIR"/*.sql; do
    if [ -f "$migration" ]; then
        echo "Running migration: $(basename "$migration")"
        psql "$DATABASE_URL" -f "$migration"
    fi
done

echo "Migrations completed successfully" 