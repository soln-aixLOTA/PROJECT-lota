#!/bin/bash

# Script for managing the Document Automation Service database

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'  # No Color

# Log functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check database connection
check_db() {
    log_info "Checking database connection..."
    if ! docker compose exec db pg_isready -U postgres &> /dev/null; then
        log_error "Database is not running"
        exit 1
    fi
}

# Create backup
create_backup() {
    local backup_dir="backups"
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="${backup_dir}/docautomation_${timestamp}.sql"
    
    mkdir -p "$backup_dir"
    
    log_info "Creating database backup..."
    docker compose exec db pg_dump -U postgres docautomation > "$backup_file"
    log_info "Backup created: $backup_file"
    
    # Compress backup
    gzip "$backup_file"
    log_info "Backup compressed: ${backup_file}.gz"
}

# Restore backup
restore_backup() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi
    
    log_warn "This will overwrite the current database. Are you sure? (y/N)"
    read -r confirm
    if [[ ! "$confirm" =~ ^[yY]$ ]]; then
        log_info "Restore cancelled"
        exit 0
    fi
    
    log_info "Restoring database from backup..."
    
    # If file is gzipped, decompress it first
    if [[ "$backup_file" == *.gz ]]; then
        gunzip < "$backup_file" | docker compose exec -T db psql -U postgres docautomation
    else
        cat "$backup_file" | docker compose exec -T db psql -U postgres docautomation
    fi
    
    log_info "Database restored successfully"
}

# Reset database
reset_db() {
    log_warn "This will delete all data and reset the database. Are you sure? (y/N)"
    read -r confirm
    if [[ ! "$confirm" =~ ^[yY]$ ]]; then
        log_info "Reset cancelled"
        exit 0
    fi
    
    log_info "Resetting database..."
    cargo sqlx database reset
    log_info "Database reset successfully"
}

# Seed test data
seed_data() {
    local count="${1:-10}"  # Default to 10 documents
    
    log_info "Seeding $count test documents..."
    
    # Create temporary script
    cat > seed-data.js << EOL
import http from 'k6/http';
import { randomString } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Load environment variables
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

export default function () {
    const headers = {
        'Authorization': \`Bearer \${AUTH_TOKEN}\`,
    };

    // Create documents with different types and classifications
    const types = ['pdf', 'doc', 'txt', 'jpg'];
    const classifications = ['legal', 'financial', 'medical', 'hr'];
    
    for (let i = 0; i < ${count}; i++) {
        const type = types[i % types.length];
        const classification = classifications[i % classifications.length];
        const content = \`Test document \${randomString(8)}\`;
        
        const data = {
            file: http.file(content, \`test-\${i}.\${type}\`, 'text/plain'),
            metadata: JSON.stringify({
                classification: classification,
                tags: [\`tag-\${i}\`, type, classification],
                custom_fields: {
                    priority: (i % 3) + 1,
                    department: classification,
                    created_by: 'seed-script'
                }
            })
        };
        
        http.post(
            \`\${BASE_URL}/documents\`,
            data,
            { headers: headers }
        );
    }
}
EOL
    
    # Run seeding script using k6
    if [ -f ".env.dev" ]; then
        source .env.dev
    fi
    
    k6 run \
        -e BASE_URL="http://localhost:8080" \
        -e AUTH_TOKEN="$AUTH_TOKEN" \
        --vus 1 --iterations 1 \
        seed-data.js
    
    rm seed-data.js
    log_info "Seeded $count test documents successfully"
}

# Show migrations status
show_migrations() {
    log_info "Checking migration status..."
    cargo sqlx migrate info
}

# Run specific migration
run_migration() {
    local migration="$1"
    
    if [ -z "$migration" ]; then
        log_error "Migration name required"
        exit 1
    fi
    
    log_info "Running migration: $migration"
    cargo sqlx migrate run --source "migrations/$migration"
}

# Show usage
show_usage() {
    echo "Usage: $0 <command> [options]"
    echo
    echo "Commands:"
    echo "  backup              Create database backup"
    echo "  restore <file>      Restore database from backup"
    echo "  reset               Reset database to clean state"
    echo "  seed [count]        Seed database with test data"
    echo "  migrations          Show migration status"
    echo "  migrate <name>      Run specific migration"
    echo "  help               Show this help message"
    echo
    exit 0
}

# Main function
main() {
    if [ $# -eq 0 ]; then
        show_usage
    fi
    
    local command="$1"
    shift
    
    case "$command" in
        backup)
            check_db
            create_backup
            ;;
        restore)
            check_db
            restore_backup "$1"
            ;;
        reset)
            check_db
            reset_db
            ;;
        seed)
            check_db
            seed_data "$1"
            ;;
        migrations)
            show_migrations
            ;;
        migrate)
            run_migration "$1"
            ;;
        help)
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            ;;
    esac
}

# Run main function with all arguments
main "$@" 