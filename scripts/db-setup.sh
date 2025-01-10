#!/bin/bash

# Create the database
docker compose exec db psql -U dev_user -c "CREATE DATABASE document_automation_dev;"

# Run migrations (if you have them)
docker compose exec app cargo sqlx database create
docker compose exec app cargo sqlx migrate run 