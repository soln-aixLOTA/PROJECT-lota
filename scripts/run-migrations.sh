#!/bin/bash

set -e

if [ -z "$DATABASE_URL" ]; then
    echo "DATABASE_URL environment variable is required"
    exit 1
fi

# Run all SQL files in the migrations directory
for file in migrations/*.sql; do
    echo "Running migration: $file"
    psql "$DATABASE_URL" -f "$file"
done

echo "Migrations completed successfully" 