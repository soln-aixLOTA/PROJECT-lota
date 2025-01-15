# Development Environment Setup Guide

This guide provides step-by-step instructions for setting up the LotaBots development environment.

## Prerequisites

### Required Software

- **Operating System**:
  - Linux (recommended)
  - macOS
  - Windows with WSL2 (Windows Subsystem for Linux)

- **Rust Toolchain**:
  - rustup (Rust toolchain manager)
  - Rust version 1.75 or later
  - cargo (Rust package manager)

- **Database**:
  - PostgreSQL 14.0 or later
  - Redis 6.0 or later

- **Containers**:
  - Docker 24.0 or later
  - Docker Compose v2.0 or later

- **Development Tools**:
  - sqlx-cli (for database migrations)
  - git (for version control)

## Installation Steps

### 1. Install Rust

```bash
# Install rustup and the Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts and select option 1 for default installation

# Verify installation
rustc --version
cargo --version

# Install sqlx-cli
cargo install sqlx-cli
```

### 2. Install PostgreSQL

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql
postgres=# CREATE DATABASE lotabots;
postgres=# CREATE USER lotabots WITH ENCRYPTED PASSWORD 'your_password';
postgres=# GRANT ALL PRIVILEGES ON DATABASE lotabots TO lotabots;
postgres=# \q
```

### 3. Install Redis

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install redis-server

# Start Redis service
sudo systemctl start redis-server
sudo systemctl enable redis-server

# Verify Redis is running
redis-cli ping  # Should return PONG
```

### 4. Install Docker and Docker Compose

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add your user to the docker group
sudo usermod -aG docker $USER

# Install Docker Compose
sudo apt install docker-compose-plugin

# Verify installations
docker --version
docker compose version
```

## Project Setup

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/lotabots.git
cd lotabots
```

### 2. Environment Configuration

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Update the following variables in `.env`:
```env
# Database Configuration
DATABASE_URL=postgresql://lotabots:your_password@localhost:5432/lotabots
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2
DB_MAX_LIFETIME_SECS=1800
DB_IDLE_TIMEOUT_SECS=300

# Server Configuration
PORT=8080
HOST=0.0.0.0
RUST_LOG=debug

# Security Configuration
# JWT Secret Management:
# For development:
#   1. Generate a secure random value:
#      JWT_SECRET=$(openssl rand -hex 32)
#   2. Add it to your .env file:
#      echo "JWT_SECRET=$JWT_SECRET" >> .env
#   3. Or export it in your shell:
#      export JWT_SECRET=$JWT_SECRET
#
# For production:
#   - NEVER commit JWT secrets to version control
#   - Use a secure secret management solution:
#     * HashiCorp Vault
#     * AWS Secrets Manager
#     * Azure Key Vault
#     * Google Cloud Secret Manager
#
JWT_EXPIRATION_HOURS=24

# Redis Configuration
REDIS_URL=redis://localhost:6379/0

# Storage Configuration
STORAGE_PATH=/path/to/your/storage
STORAGE_MAX_FILE_SIZE=10485760  # 10MB
```

### 3. Database Setup

```bash
# Run database migrations
sqlx database create
sqlx migrate run

# Verify migrations
sqlx migrate info
```

### 4. Build and Run (Local Development)

```bash
# Build the project
cargo build

# Run the project
cargo run

# Run with specific features (if any)
cargo run --features "feature1 feature2"
```

### 5. Run with Docker Compose

```bash
# Build and start all services
docker compose up --build -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

## Verification

1. Check if the API is running:
```bash
curl http://localhost:8080/health
```

2. Register a test user:
```bash
curl -X POST http://localhost:8080/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "TestPassword123!",
    "role": "user"
  }'
```

## Common Issues and Solutions

### Database Connection Issues

1. **Error**: "Failed to connect to PostgreSQL"
   - Verify PostgreSQL is running: `sudo systemctl status postgresql`
   - Check connection string in `.env`
   - Ensure database and user exist with correct permissions

2. **Error**: "Redis connection refused"
   - Verify Redis is running: `sudo systemctl status redis-server`
   - Check Redis URL in `.env`

### Build Issues

1. **Error**: "Missing system libraries"
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential pkg-config libssl-dev
   ```

2. **Error**: "Failed to compile"
   - Update Rust: `rustup update`
   - Clean build: `cargo clean && cargo build`

### Docker Issues

1. **Error**: "Permission denied"
   - Ensure user is in docker group: `groups $USER`
   - Relogin or restart system after adding to group

2. **Error**: "Port already in use"
   - Check for running processes: `sudo lsof -i :8080`
   - Stop conflicting process or change port in `.env`

## Development Tools

### Recommended VSCode Extensions

- rust-analyzer
- Better TOML
- Docker
- GitLens
- SQLTools

### Useful Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check for security vulnerabilities
cargo audit
```

## Next Steps

- Review the [API Documentation](./openapi.yaml)
- Explore the [Architecture Documentation](./ARCHITECTURE.md)
- Set up your IDE with recommended extensions
- Join the development chat (if available)

## Support

For additional help:
- Create an issue in the repository
- Contact the development team
- Check the troubleshooting guide

## Environment Variables and Secrets

For local development:

1. Copy `.env.example` to `.env`
2. Generate secure random values for secrets:
   ```bash
   # On Linux/Mac
   echo "JWT_SECRET=$(openssl rand -hex 32)" >> .env
   ```
3. Never commit `.env` file or actual secret values to version control
4. For production, use secure secret management solutions (e.g., AWS Secrets Manager, HashiCorp Vault)

### Setting up Secrets

For local development:

1. Generate a secure JWT secret:
```bash
# Generate a secure random secret
# DO NOT commit the actual value to version control!
JWT_SECRET=$(openssl rand -hex 32)

# Add it to your .env file (make sure .env is in .gitignore!)
echo "JWT_SECRET=DO_NOT_USE" >> .env  # First add placeholder
sed -i "s/DO_NOT_USE/$JWT_SECRET/" .env  # Then replace it with actual value

# Or export it directly in your shell (preferred for development)
export JWT_SECRET=$JWT_SECRET
```

⚠️ **IMPORTANT**:
- Never commit the actual JWT secret to version control
- Use different secrets for development, testing, and production
- In production, use a secrets management solution (e.g., AWS Secrets Manager, HashiCorp Vault)
