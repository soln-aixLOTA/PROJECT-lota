#!/bin/bash

# Load environment variables
source .env

# Create test database
psql -U postgres -c "DROP DATABASE IF EXISTS lotabots_test;"
psql -U postgres -c "CREATE DATABASE lotabots_test;"

# Run migrations
DATABASE_URL=$DATABASE_URL sqlx migrate run 