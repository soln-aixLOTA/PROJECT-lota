# LotaBots Platform - Troubleshooting Guide

This guide addresses common issues you might encounter while setting up the LotaBots Platform and provides solutions for each problem.

## Setup Script Issues

### 1. Permission Denied Error

**Symptom:**
```
Please run as root or with sudo
Failed to execute scripts/setup.sh
```

**Cause:**
The setup script requires root privileges to install system packages and configure services. This error occurs when running the script directly without sudo. Administrative privileges are needed for:
- Installing system packages (PostgreSQL, Redis, etc.)
- Configuring system services
- Creating and modifying system directories

**Solution:**
Run the setup script with sudo:
```bash
sudo ./setup.sh
```

### 2. Missing Execute Permissions on Scripts

**Symptom:**
The setup might start running, but then fail with a "Permission denied" error when trying to execute a script within the `scripts/` directory. You might see an error like:
```
/scripts/configure_env.sh: Permission denied
Failed to execute scripts/configure_env.sh
```

You can check script permissions with:
```bash
ls -la scripts/
```
Look for the `x` permission flag in the output (e.g., `-rwxr-xr-x`).

**Cause:**
Script files in the `scripts/` directory don't have execute permissions set. All scripts need execute permissions (`+x`) to run properly.

**Solution:**
Grant execute permissions to the necessary scripts:
```bash
chmod +x scripts/setup.sh scripts/configure_env.sh scripts/db_setup.sh scripts/deploy.sh
```

### 3. Missing Database Connection

**Symptom:**
```
Error: Could not connect to database
Please check your database configuration in .env
```

**Cause:**
PostgreSQL server is not installed or running on the system. The setup script reads database connection details from the `.env` file but cannot establish a connection if PostgreSQL isn't properly installed and configured.

**Solution:**
1. Install PostgreSQL server and its dependencies:
```bash
sudo apt-get update && sudo apt-get install -y postgresql postgresql-contrib
```

2. The setup will automatically create the required database and user with these default settings (configured in `.env`):
- Database name: lotabots
- Username: postgres
- Password: postgres
- Host: localhost
- Port: 5432

3. Verify PostgreSQL is running and listening:
```bash
# Check service status
sudo systemctl status postgresql

# Verify PostgreSQL is listening on port 5432
sudo ss -tuln | grep 5432
```

### 4. Cargo/Rust Not Found

**Symptom:**
```
./scripts/db_setup.sh: line 47: cargo: command not found
```

**Cause:**
Rust and Cargo are not properly installed or not in the system PATH when running with sudo. This happens because sudo runs commands in a different environment, and the root user needs its own Rust installation.

**Solution:**
1. Install Rust for the root user:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo sh -s -- -y
```

2. Source the Rust environment when running scripts that need it:
```bash
sudo bash -c 'source /root/.cargo/env && ./scripts/db_setup.sh'
```

Alternative: Add Rust to the system-wide PATH by adding these lines to `/etc/profile`:
```bash
export PATH="$PATH:/root/.cargo/bin"
source "/root/.cargo/env"
```

### 5. Kubernetes Connection Issues

**Symptom:**
```
error: error validating "STDIN": error validating data: failed to download openapi: Get "http://localhost:8080/openapi/v2?timeout=32s": dial tcp 127.0.0.1:8080: connect: connection refused
```

**Cause:**
The kubectl client is trying to connect to a Kubernetes API server that is not running or accessible. This typically means either:
- Kubernetes cluster is not properly configured
- API server is not running
- kubectl configuration is incorrect

**Solution:**
1. Skip Kubernetes secret creation during initial setup by answering 'N' when prompted.
2. Configure your Kubernetes cluster separately following your organization's guidelines.
3. Verify your kubectl configuration:
```bash
# Check current context
kubectl config current-context

# Review full configuration
kubectl config view
```
4. Run the deployment script later when Kubernetes is properly configured:
```bash
./scripts/deploy.sh
```

### 6. Interactive Input Issues

**Symptom:**
Setup script prompts for multiple configuration values interactively, making automation difficult.

**Cause:**
The setup script expects interactive input for various configuration values like Vault address, database credentials, etc.

**Solution:**
You can provide all inputs at once using echo (for development/testing only):
```bash
echo -e "http://vault.lotabots.svc:8200\napi-gateway\nlotabots\nlocalhost\n5432\nlotabots\npostgres\npostgres\nlocalhost\n6379\n\nN" | sudo ./setup.sh
```

⚠️ **Security Warning:** Do not use this method in production as it exposes sensitive information in command history and process lists. Instead:
- Use environment variables
- Use a secure secrets management solution (like HashiCorp Vault)
- Configure values interactively in a secure environment

## Prerequisites Checklist

Before running the setup script, ensure you have:

1. Root/sudo access to the system
2. Internet connection for package installation
3. System Requirements:
   - Disk space: at least 500MB for installation
   - Memory: minimum 2GB recommended for running services
4. Required software versions:
   - Docker 27.4.0 or later (for container support)
   - Rust 1.84.0 or later (required for building components)
   - PostgreSQL 14 or later (for database functionality)
   - Redis 6.0 or later (for caching and message queues)
5. Basic understanding of:
   - PostgreSQL (database operations)
   - Redis (caching and queues)
   - Rust/Cargo (build system)
   - Kubernetes (if deploying)

## Environment Configuration

The setup script will create a `.env` file with default settings. Review and adjust these settings according to your environment:

```env
# Vault Configuration
VAULT_ADDR=http://vault.lotabots.svc:8200  # HashiCorp Vault server address
VAULT_ROLE=api-gateway                      # Role for Vault authentication
VAULT_NAMESPACE=lotabots                    # Vault namespace for secrets isolation

# PostgreSQL Configuration
POSTGRES_HOST=localhost                     # Database server hostname/IP
POSTGRES_PORT=5432                         # PostgreSQL default port
POSTGRES_DB=lotabots                       # Application database name
POSTGRES_USER=postgres                     # Database admin username
POSTGRES_PASSWORD=postgres                 # Database admin password

# Redis Configuration
REDIS_HOST=localhost                       # Redis server hostname/IP
REDIS_PORT=6379                           # Redis default port
REDIS_PASSWORD=                           # Redis auth password (if enabled)
```

## Setup Process Overview

The setup script performs these steps in order:

1. Installs system prerequisites
   - Rust toolchain (for building components)
   - Docker (for containerization)
   - kubectl (for Kubernetes deployment)
   - PostgreSQL client (for database operations)
   - Redis client (for cache operations)
2. Configures the environment
   - Creates `.env` file with default values
   - Sets up configuration based on user input
3. Sets up the database
   - Creates database and user if they don't exist
   - Runs all pending migrations
4. Optionally deploys to Kubernetes

**Important:** These steps must be executed in order. Deviating from this sequence may lead to incomplete setup or configuration errors.

## Additional Notes

- The setup script is idempotent - you can run it multiple times safely
- Database migrations are automatically handled during setup
- For security in production:
  - Change all default passwords immediately after setup
  - Use secure secret management (e.g., HashiCorp Vault)
  - Configure proper access controls and firewalls
  - Enable SSL/TLS for all services
- Keep the `.env` file secure and never commit it to version control
- Consider using configuration management tools (e.g., Ansible, Puppet) for production deployments
- Monitor logs during setup for any warnings or errors:
  ```bash
  journalctl -fu postgresql  # PostgreSQL logs
  journalctl -fu redis      # Redis logs
  ```
