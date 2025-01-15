# LotaBots Codebase Documentation

## Overview

LotaBots is a platform for hardware and AI model attestation, built with Rust using a microservices architecture. The platform provides secure verification of GPU hardware, AI models, and resource management capabilities.

## Project Structure

```
services/
├── api-gateway/        # API Gateway service
├── auth/              # Authentication service
├── hardware-attestation/ # Hardware attestation service
├── ai-attestation/    # AI model attestation service
├── document-automation/ # Document management service
└── resource-management/ # Resource management service

shared/
├── config/            # Configuration management
├── db/               # Database utilities
├── error/            # Error types and handling
├── models/           # Shared data models
├── testing/          # Testing utilities
└── utils/            # Common utilities

infrastructure/
├── terraform/        # Infrastructure as Code
└── kubernetes/       # Kubernetes manifests

docs/
├── architecture/     # Architecture documentation
├── deployment/       # Deployment guides
└── development/      # Development guides
```

## Services

### API Gateway Service
- Entry point for all client requests
- Authentication and authorization
- Request routing
- Rate limiting
- Usage tracking

### Authentication Service
- User registration and login
- Password management
- Token generation and validation
- Role-based access control

### Hardware Attestation Service
- GPU capability verification
- Security feature validation
- Resource monitoring
- Container isolation checks

### AI Attestation Service
- Model verification
- Compliance checking
- Behavioral attestation
- Audit trail generation

### Document Automation Service
- Document creation and updates
- Metadata management
- File storage integration
- Access control

### Resource Management Service
- GPU resource scheduling
- Container management
- Memory allocation
- Resource monitoring

## Shared Libraries

### Config Library (`shared/config`)
- Configuration loading and validation
- Environment variable handling
- Default configurations
- Configuration types

### Database Library (`shared/db`)
- Database connection management
- Migration handling
- Common database operations
- Connection pooling

### Error Library (`shared/error`)
- Common error types
- Error handling utilities
- HTTP error responses
- Error logging

### Models Library (`shared/models`)
- Shared data structures
- Database models
- API request/response types
- Validation logic

### Testing Library (`shared/testing`)
- Test utilities
- Fixtures and factories
- Mock implementations
- Test database setup

### Utils Library (`shared/utils`)
- Logging utilities
- Metrics collection
- Distributed tracing
- Common helper functions

## Infrastructure

### Terraform
- AWS infrastructure definitions
- Network configuration
- Service deployment
- Resource provisioning

### Kubernetes
- Service deployments
- Configuration management
- Resource allocation
- Service discovery

## Development Guidelines

1. **Code Style**
   - Follow Rust style guide
   - Use `cargo fmt` for formatting
   - Run `cargo clippy` for linting
   - Document public APIs

2. **Git Workflow**
   - Feature branch workflow
   - Pull request reviews
   - Semantic versioning
   - Signed commits

3. **Testing**
   - Unit tests for all components
   - Integration tests for services
   - End-to-end tests for workflows
   - Performance testing

4. **Documentation**
   - Keep API docs updated
   - Document breaking changes
   - Maintain changelog
   - Update architecture docs

## Security Best Practices

1. **Authentication**
   - JWT-based authentication
   - Secure password hashing
   - Token refresh mechanism
   - Rate limiting

2. **Authorization**
   - Role-based access control
   - Resource-level permissions
   - Audit logging
   - Access reviews

3. **Data Security**
   - Encryption at rest
   - Secure communication
   - Key management
   - Regular security audits

## Monitoring and Observability

1. **Metrics**
   - Request latency
   - Error rates
   - Resource utilization
   - Business metrics

2. **Logging**
   - Structured logging
   - Log aggregation
   - Error tracking
   - Audit trails

3. **Tracing**
   - Distributed tracing
   - Request tracking
   - Performance monitoring
   - Dependency analysis
