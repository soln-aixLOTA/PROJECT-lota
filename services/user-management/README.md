# User Management Service

A microservice responsible for managing users, tenants, authentication, and authorization in the LotaBots platform.

## Features

- User management (CRUD operations)
- Tenant management (multi-tenancy support)
- Authentication (JWT-based)
- Authorization (Role-based access control)
- Password hashing with Argon2
- Rate limiting
- Audit logging

## Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Redis 6+
- Docker & Docker Compose (for local development)

## Local Development

1. **Start the Development Environment**

   ```bash
   # Start all required services
   docker-compose up
   ```

2. **Run Migrations**

   ```bash
   # Using SQLx CLI
   sqlx migrate run
   ```

3. **Run Tests**

   ```bash
   # Run all tests
   cargo test

   # Run specific test
   cargo test test_create_user
   ```

## API Endpoints

### Authentication

- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/refresh` - Refresh access token

### Users

- `POST /api/v1/users` - Create user
- `GET /api/v1/users` - List users
- `GET /api/v1/users/{id}` - Get user
- `PUT /api/v1/users/{id}` - Update user
- `DELETE /api/v1/users/{id}` - Delete user

### Tenants

- `POST /api/v1/tenants` - Create tenant
- `GET /api/v1/tenants` - List tenants
- `GET /api/v1/tenants/{id}` - Get tenant
- `PUT /api/v1/tenants/{id}` - Update tenant
- `DELETE /api/v1/tenants/{id}` - Delete tenant

## Configuration

Configuration is managed through YAML files and environment variables:

- `config/base.yaml` - Base configuration
- `config/development.yaml` - Development overrides
- `config/production.yaml` - Production overrides

Environment variables override file-based configuration. See `config/base.yaml` for available options.

## Database Schema

The service uses PostgreSQL with the following main tables:

- `users` - User accounts
- `tenants` - Multi-tenant organizations
- `roles` - User roles
- `permissions` - Role permissions
- `user_roles` - User-role associations
- `role_permissions` - Role-permission associations

See `migrations/` for complete schema details.

## Architecture

The service follows a clean architecture pattern:

```
src/
├── main.rs           # Application entry point
├── config.rs         # Configuration management
├── db.rs             # Database connection and migrations
├── error.rs          # Error types and handling
├── models/           # Domain models
├── handlers/         # HTTP request handlers
└── middleware/       # HTTP middleware
```

## Security

- Passwords are hashed using Argon2
- JWT tokens for authentication
- Rate limiting per client
- Input validation and sanitization
- SQL injection protection via SQLx
- CORS configuration
- Audit logging

## Monitoring

The service exposes:

- Health check endpoint: `/health`
- Metrics endpoint: `/metrics`
- Structured JSON logging

## Contributing

1. Create a feature branch
2. Make changes
3. Add tests
4. Run `cargo fmt` and `cargo clippy`
5. Create pull request

## License

[License details here]
