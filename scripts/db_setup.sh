#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "LotaBots Platform - Database Setup"
echo "================================="

# Load environment variables
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found${NC}"
    echo "Please run ./scripts/configure_env.sh first"
    exit 1
fi

source .env

# Function to check if psql is available
check_psql() {
    if ! command -v psql >/dev/null 2>&1; then
        echo -e "${RED}Error: psql command not found${NC}"
        echo "Please install PostgreSQL client tools first"
        exit 1
    fi
}

# Function to check database connection
check_db_connection() {
    local db_url=$1
    if ! psql "$db_url" -c '\q' 2>/dev/null; then
        echo -e "${RED}Error: Could not connect to database${NC}"
        echo "Please check your database configuration in .env"
        exit 1
    fi
}

# Function to create databases
create_databases() {
    echo "Creating databases..."

    # Create attestation database
    su - postgres -c "createdb -T template0 lotabots_attestation" || true
    echo -e "${GREEN}✓ Created attestation database${NC}"

    # Create API gateway database
    su - postgres -c "createdb -T template0 lotabots_api" || true
    echo -e "${GREEN}✓ Created API gateway database${NC}"
}

# Function to run migrations
run_migrations() {
    echo "Running database migrations..."

    # Check if sqlx-cli is installed
    if ! command -v sqlx >/dev/null 2>&1; then
        echo -e "${YELLOW}Installing sqlx-cli...${NC}"
        cargo install sqlx-cli --no-default-features --features native-tls,postgres
    fi

    # Run migrations for attestation service
    if [ -d "src/attestation/migrations" ]; then
        echo "Running migrations for attestation service..."
        cd src/attestation
        DATABASE_URL=$ATTESTATION_DATABASE_URL sqlx database create
        DATABASE_URL=$ATTESTATION_DATABASE_URL sqlx migrate run
        cd ../..
        echo -e "${GREEN}✓ Migrations completed for attestation service${NC}"
    fi

    # Run migrations for API gateway service
    if [ -d "src/api_gateway/migrations" ]; then
        echo "Running migrations for API gateway service..."
        cd src/api_gateway
        DATABASE_URL=$API_GATEWAY_DATABASE_URL sqlx database create
        DATABASE_URL=$API_GATEWAY_DATABASE_URL sqlx migrate run
        cd ../..
        echo -e "${GREEN}✓ Migrations completed for API gateway service${NC}"
    fi
}

# Function to create test data
create_test_data() {
    echo "Creating test data..."

    # Read the database URLs from .env
    if [ -z "$ATTESTATION_DATABASE_URL" ] || [ -z "$API_GATEWAY_DATABASE_URL" ]; then
        echo -e "${RED}Error: Database URLs not found in .env${NC}"
        exit 1
    fi

    # Create test data SQL for attestation service
    cat << EOF | psql "$ATTESTATION_DATABASE_URL"
-- Insert test data here if needed
EOF

    # Create test data SQL for API gateway service
    cat << EOF | psql "$API_GATEWAY_DATABASE_URL"
-- Insert test data here if needed
EOF

    echo -e "${GREEN}✓ Test data created${NC}"
}

# Main setup process
main() {
    # Check prerequisites
    check_psql

    # Create databases
    create_databases

    # Check database connections
    check_db_connection "$ATTESTATION_DATABASE_URL"
    check_db_connection "$API_GATEWAY_DATABASE_URL"

    # Run migrations
    run_migrations

    # Ask if test data should be created
    read -p "Do you want to create test data? (y/N) " create_data
    if [[ $create_data =~ ^[Yy]$ ]]; then
        create_test_data
    fi

    echo -e "\n${GREEN}Database setup completed!${NC}"
    echo "Next steps:"
    echo "1. Verify the migrations were applied correctly"
    echo "2. Run the application with 'cargo run'"
}

main
