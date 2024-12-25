# User Management Service

## Overview

The User Management Service is a core component of the LotaBots platform, providing enterprise-grade user authentication, authorization, and multi-tenant management capabilities. Built with Rust using the Actix-web framework, it offers high performance, security, and scalability.

## Features

- **Multi-tenancy Support**
  - Tenant isolation
  - Custom domains
  - Subscription tiers (Free, Professional, Enterprise, Custom)
  - Resource quotas and limits

- **User Management**
  - User registration and authentication
  - Role-based access control (RBAC)
  - Password policies and MFA support
  - User profile management

- **Enterprise Security**
  - JWT-based authentication
  - Rate limiting
  - Audit logging
  - Request validation

- **Monitoring and Metrics**
  - Prometheus metrics
  - Request tracing
  - Performance monitoring
  - Usage analytics

## Prerequisites

- Rust 1.70 or higher
- PostgreSQL 14 or higher
- Docker (optional)
- NVIDIA GPU (optional, for GPU quota management)

## Getting Started

1. **Clone the Repository**
   ```bash
   git clone https://github.com/lotabots/user-management.git
   cd user-management
   ```

2. **Set Up Environment Variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Set Up Database**
   ```bash
   # Create database
   createdb lotabots_user_management

   # Run migrations
   sqlx migrate run
   ```

4. **Build and Run**
   ```bash
   # Development
   cargo run

   # Production
   cargo build --release
   ./target/release/user-management
   ```

5. **Run Tests**
   ```bash
   # Unit tests
   cargo test

   # Integration tests
   cargo test --features test-integration

   # All tests with coverage
   cargo tarpaulin
   ```

## Configuration

The service can be configured through environment variables:

```env
# Server
HOST=127.0.0.1
PORT=8080
RUST_LOG=info

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/lotabots_user_management

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRATION=3600

# Rate Limiting
RATE_LIMIT_BURST=100
RATE_LIMIT_REPLENISH=10

# Metrics
METRICS_PORT=9090
```

## API Documentation

See [docs/tenant-api.md](docs/tenant-api.md) for detailed API documentation.

## Architecture

### Components

1. **HTTP Server**
   - Actix-web framework
   - RESTful API endpoints
   - Middleware chain

2. **Middleware**
   - Tenant identification
   - Rate limiting
   - Metrics collection
   - Audit logging

3. **Services**
   - Tenant management
   - User authentication
   - Role management
   - Resource tracking

4. **Database**
   - PostgreSQL with SQLx
   - Migrations
   - Connection pooling

### Directory Structure

```
user-management/
├── src/
│   ├── main.rs              # Application entry point
│   ├── error.rs             # Error types
│   ├── handlers/            # HTTP handlers
│   ├── middleware/          # Custom middleware
│   ├── models/              # Data models
│   ├── repositories/        # Database access
│   └── services/           # Business logic
├── migrations/              # Database migrations
├── tests/                   # Test suites
├── docs/                    # Documentation
└── Cargo.toml              # Dependencies
```

## Deployment

### Docker

```bash
# Build image
docker build -t lotabots/user-management .

# Run container
docker run -p 8080:8080 \
  --env-file .env \
  lotabots/user-management
```

### Kubernetes

```bash
# Apply configuration
kubectl apply -f k8s/

# Check status
kubectl get pods -n lotabots
```

## Monitoring

### Prometheus Metrics

The service exposes metrics at `/metrics` in Prometheus format:

- `http_requests_total`: Request count by tenant and endpoint
- `http_request_duration_seconds`: Request latency
- `gpu_time_seconds`: GPU processing time
- `data_processed_bytes`: Data transfer volume

### Health Check

A health check endpoint is available at `/health`:

```bash
curl http://localhost:8080/health
```

## Development

### Code Style

```bash
# Format code
cargo fmt

# Check lints
cargo clippy
```

### Adding Migrations

```bash
# Create migration
sqlx migrate add create_users_table

# Run migrations
sqlx migrate run

# Revert migration
sqlx migrate revert
```

### Running Integration Tests

```bash
# Start test database
docker-compose up -d db-test

# Run tests
cargo test --features test-integration
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

Copyright © 2024 LotaBots. All rights reserved.

## Support

For support, contact support@lotabots.ai or open an issue on GitHub. 

## API Specifications

### User Registration

**Endpoint:** `POST /api/v1/users/register`

**Request Body:**

```json
{
  "tenant_id": "uuid", // Tenant ID
  "email": "[email address removed]", // User's email address
  "password": "StrongPassword123!", // User's password
  "first_name": "John", // User's first name (optional)
  "last_name": "Doe", // User's last name (optional)
  "mfa_enabled": false // Whether to enable MFA (optional, default: false)
}
```

**Response (201 Created):**

```json
{
  "id": "uuid", // User ID
  "tenant_id": "uuid",
  "email": "[email address removed]",
  "first_name": "John",
  "last_name": "Doe",
  "created_at": "2024-01-01T10:00:00Z",
  "updated_at": "2024-01-01T10:00:00Z",
  "last_login_at": null,
  "status": "active",
  "mfa_enabled": false
}
```

**Error Responses:**

* 400 Bad Request: Invalid input data (e.g., invalid email format, password too short).
  ```json
  {
      "code": 400,
      "message": "Invalid email format"
  }
  ```
* 409 Conflict: Email already exists within the tenant.
  ```json
  {
      "code": 409,
      "message": "User already exists with email: [email address removed]"
  }
  ```
* 500 Internal Server Error: Database error or other unexpected error.

### User Login

**Endpoint:** `POST /api/v1/auth/login`

**Request Body:**

```json
{
  "email": "[email address removed]",
  "password": "StrongPassword123!",
  "mfa_code": "123456" // Optional MFA code
}
```

**Response (200 OK):**

```json
{
  "access_token": "jwt_access_token",
  "refresh_token": "jwt_refresh_token",
  "token_type": "bearer",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "email": "[email address removed]",
    // ... other user profile fields ...
    "roles": ["user", "some_role"],
    "permissions": ["user.read", "bot.create"]
  }
}
```

**Error Responses:**

* 401 Unauthorized: Invalid credentials or account locked.
* 401 Unauthorized: MFA required.
* 401 Unauthorized: Invalid MFA code.
* 500 Internal Server Error: Unexpected error. 

## Data Flow Diagrams

```plantuml
@startuml
!include [invalid URL removed]

System_Boundary(c1, "LotaBots Platform") {
  Rel(API Gateway, User Management Service, "Authenticates users, Authorizes requests", "HTTPS/REST")
  Rel(User Management Service, Database, "Reads/Writes user data", "SQL")
  Rel(User Management Service, Audit Log Service, "Logs security events", "gRPC")
  Rel(User Management Service, Billing Service, "Triggers billing events", "Async Message Queue")

  Component(API Gateway, "API Gateway", "Rust, Actix Web", "Handles API requests, authentication, routing, and load balancing")
  Component(User Management Service, "User Management Service", "Rust, Actix Web", "Manages user accounts, authentication, authorization, and profile management")
  ComponentDb(Database, "Database", "PostgreSQL", "Stores user data, roles, permissions, and tenant information")
  Component(Audit Log Service, "Audit Log Service", "Go", "Stores audit logs")
  Component(Billing Service, "Billing Service", "Python", "Handles billing and invoicing")

}
@enduml
```

## Algorithms and Data Structures

- **Password Hashing**: Argon2id with a memory cost of 64 MiB, time cost of 3 iterations, and parallelism factor of 4. A randomly generated 32-byte salt is used for each password.
- **User Lookup**: Optimized using a unique B-tree index on the `(tenant_id, email)` columns.
- **JWTs**: Short expiration time (e.g., 15 minutes) with refresh tokens for session management.
- **Caching**: Frequently accessed data (e.g., user roles and permissions) cached in a distributed cache (e.g., Redis) for performance optimization.

## Error Handling and Logging

- **Error Handling**: The `ServiceError` enum is used to represent different types of errors in the service layer. Error responses are returned in a consistent JSON format:

  ```json
  {
      "code": 400,
      "message": "Invalid request: Email is required"
  }
  ```

- **Logging**: The `tracing` crate is used for logging. Log levels are used as follows:
  - **DEBUG**: Detailed debugging information.
  - **INFO**: General information about the service's operation.
  - **WARN**: Warnings about potential issues.
  - **ERROR**: Errors that prevent the service from functioning correctly.

- **Log Messages**: Examples of log messages for different events:
  - User registration: `INFO: User registered with email: [email address removed]`
  - Login attempt: `INFO: Login attempt for user: [email address removed]`
  - Password change: `INFO: Password changed for user: [email address removed]`
  - Failed login: `WARN: Failed login attempt for user: [email address removed]`
  - Role assignment: `INFO: Role assigned to user: [email address removed]`

## Security Design

### Authentication

- **JWT (JSON Web Tokens)**: Used for authentication.
  - **Claims**:
    - `sub`: User ID (UUID)
    - `tenant_id`: Tenant ID (UUID)
    - `roles`: List of user's roles
    - `permissions`: List of user's permissions
    - `exp`: Expiration time (timestamp)
    - `iat`: Issued at time (timestamp)
  - **Secret Management**: JWT secret stored securely in Kubernetes Secrets as `user-management-secrets` and injected as an environment variable.
  - **Expiration**: Access tokens have a short expiration time (e.g., 1 hour). Refresh tokens have a longer expiration time (e.g., 7 days).
  - **Token Refresh**: `/api/v1/auth/refresh` endpoint used to obtain a new access token using a valid refresh token.

### Authorization

- **Role-Based Access Control (RBAC)**:
  - Users can be assigned multiple roles.
  - Roles have associated permissions.
  - Permissions define actions on specific resources (e.g., `user.create`, `bot.read`).
- **Permission Checks**:
  - `PermissionService` provides methods to check if a user has a specific permission or a set of permissions.
  - Checks performed in the service layer before executing sensitive operations.

### Input Validation

- All user inputs validated using the `validator` crate to prevent common vulnerabilities:
  - Email addresses validated using a regular expression or a dedicated email validation library.
  - Password complexity enforced (minimum length, uppercase, lowercase, numbers, special characters).
  - String inputs validated for length and allowed characters.
  - UUIDs validated using the `uuid` crate.

## Performance Considerations

- **Database Optimizations**: Queries optimized using indexes and query planning tools.
- **Caching**: Frequently accessed data (e.g., user roles and permissions) cached in a distributed cache (e.g., Redis) to improve performance.
- **Connection Pooling**: Database connection pooling configured to manage and reuse connections efficiently, reducing latency and improving throughput.

## Scalability Considerations

- **Kubernetes Deployment**: The service is deployed in a Kubernetes environment to support horizontal scaling.
- **Horizontal Pod Autoscaler**: Used to automatically adjust the number of pods in response to changes in load.
- **Readiness/Liveness Probes**: Configured to ensure that the service is running and healthy, allowing Kubernetes to manage pod restarts and scaling effectively.

## Integration with Other Services

- **API Gateway**: Handles authentication and authorization flow, forwarding requests to the User Management Service after validating JWTs and extracting the `tenant_id`.
- **Audit Log Service**: Logs security-related events, such as user registration, login, and role assignments, via a message queue (e.g., RabbitMQ or Kafka).
- **Billing System (Future)**: User creation/updates trigger billing events to track subscription usage and manage billing processes.

## Multi-Tenancy Considerations

- **Tenant Isolation**: Enforced at both the service and database levels to ensure data privacy and security.
- **Tenant-Specific Configurations**: Managed through configuration files or environment variables, allowing customization of password policies, SSO settings, and other tenant-specific features.