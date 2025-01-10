# AI Attestation Service

## Overview

The AI Attestation Service is a web service responsible for verifying and attesting AI model behavior, safety, and compliance within the LotaBots platform. It provides RESTful APIs for model verification, monitoring, and compliance tracking.

## Features

- RESTful API endpoints using Axum
- PostgreSQL database integration
- OpenTelemetry integration for observability
- Structured logging and tracing
- Cookie-based authentication
- Comprehensive error handling

## Architecture

The service follows a clean architecture pattern with the following components:

- `api/`: REST API endpoints and handlers
- `config/`: Configuration management
- `constants/`: System-wide constants
- `error/`: Error types and handling
- `models/`: Data models and schemas
- `repository/`: Database access layer
- `services/`: Business logic implementation

## Dependencies

- Axum for web framework
- SQLx for database operations
- OpenTelemetry for observability
- Tokio for async runtime
- Various utility crates for serialization, validation, etc.

## Configuration

The service is configured through environment variables and configuration files. Key configuration options include:

- Database connection settings
- API endpoints and ports
- Authentication settings
- Telemetry configuration

## Development

```bash
# Build the service
cargo build

# Run tests
cargo test

# Run the service
cargo run
```

## API Documentation

API documentation is available through OpenAPI/Swagger. Start the service and visit `/docs` for interactive documentation.
