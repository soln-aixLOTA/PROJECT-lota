# LotaBots Codebase Documentation

## Overview

LotaBots is a platform for hardware and AI model attestation, built with Rust using a microservices architecture. The platform provides secure verification of GPU hardware, AI models, and resource management capabilities.

## Project Structure

```
src/
├── api/              # API endpoint definitions
├── api_gateway/      # API Gateway service
├── attestation/      # Hardware and AI attestation service
├── auth/            # Authentication and authorization
├── backend/         # Core backend services
├── components/      # Reusable components
├── config/          # Configuration management
├── core/            # Core business logic
├── db/              # Database access and models
├── frontend/        # Web interface
├── handlers/        # Request handlers
├── lib/             # Shared libraries
├── middleware/      # Web middleware components
├── mlops/           # ML operations and management
├── models/          # Data models and schemas
├── routes/          # Route definitions
├── services/        # Business services
├── storage/         # Storage management
├── user_management/ # User management service
└── utils/           # Utility functions
```

## Core Components

### 1. API Gateway Service

The API Gateway serves as the main entry point for all client requests, handling:
- Authentication and authorization
- Request routing
- Rate limiting
- Request/Response transformation

**Key Files**:
- `src/api_gateway/main.rs`: Service entry point
- `src/middleware/rate_limit.rs`: Rate limiting implementation
- `src/middleware/auth.rs`: Authentication middleware

### 2. Attestation Service

Handles hardware and AI model verification:

#### Hardware Attestation
- GPU capability verification
- Security feature validation
- Resource monitoring
- Container isolation checks

#### AI Model Attestation
- Model verification
- Compliance checking
- Behavioral attestation
- Audit trail generation

**Key Files**:
- `src/attestation/src/main.rs`: Attestation service entry point
- `src/attestation/migrations/`: Database migrations for attestation records

### 3. Authentication System

Implements secure user authentication and authorization:
- JWT-based authentication
- Role-based access control (RBAC)
- Token refresh mechanism
- Password hashing with bcrypt

**Key Files**:
- `src/auth/mod.rs`: Authentication module
- `src/models/auth.rs`: Authentication data models
- `src/handlers/auth.rs`: Authentication endpoints

### 4. Document Management

Handles document processing and storage:
- Document creation and updates
- Metadata management
- File storage integration
- Access control

**Key Files**:
- `src/handlers/documents.rs`: Document handling endpoints
- `src/models/document.rs`: Document data models

### 5. Resource Management

Manages compute resource allocation:
- GPU resource scheduling
- Container management
- Memory allocation
- Resource monitoring

**Key Files**:
- `src/resource_management.rs`: Resource management implementation

## Data Models

### Core Data Models

1. **User Model**
```rust
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    // ... other fields
}
```

2. **Document Model**
```rust
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub content_type: String,
    pub metadata: JsonValue,
    // ... other fields
}
```

3. **Attestation Model**
```rust
pub struct Attestation {
    pub id: Uuid,
    pub hardware_id: String,
    pub attestation_type: String,
    pub status: String,
    pub metadata: JsonValue,
    // ... other fields
}
```

## Database Schema

The system uses PostgreSQL with the following main tables:

1. **users**
   - Primary key: id (UUID)
   - Unique constraints: username, email
   - Indexes: username, email, role

2. **documents**
   - Primary key: id (UUID)
   - Foreign key: user_id references users(id)
   - Indexes: user_id, document_type

3. **attestations**
   - Primary key: id (UUID)
   - Indexes: hardware_id, attestation_type, status

## Middleware Components

1. **Rate Limiting**
   - Token bucket algorithm
   - Per-IP and per-user limits
   - Redis-backed storage

2. **Authentication**
   - JWT validation
   - Role verification
   - Token refresh handling

3. **Request ID**
   - Unique request tracking
   - Logging correlation
   - Debug tracing

4. **Security Headers**
   - CORS configuration
   - Content Security Policy
   - XSS protection

## Error Handling

The system uses a centralized error handling approach:

```rust
pub enum AppError {
    Authentication(String),
    Authorization(String),
    NotFound(String),
    ValidationError(String),
    DatabaseError(String),
    Internal(String),
}
```

Error responses follow a consistent format:
```json
{
    "message": "Error description",
    "code": "ERROR_CODE",
    "details": { ... }
}
```

## Configuration Management

Configuration is managed through:
1. Environment variables
2. Configuration files
3. Runtime configuration

Key configuration areas:
- Database connections
- Redis settings
- JWT secrets
- Rate limiting parameters
- Storage paths
- Logging levels

## Testing Strategy

1. **Unit Tests**
   - Individual component testing
   - Mocked dependencies
   - Property-based testing

2. **Integration Tests**
   - API endpoint testing
   - Database integration
   - Service communication

3. **Performance Tests**
   - Load testing
   - Stress testing
   - Resource utilization

## Deployment

The application supports multiple deployment options:

1. **Docker Deployment**
   - Multi-stage builds
   - Container orchestration
   - Resource isolation

2. **Kubernetes Deployment**
   - Service scaling
   - Load balancing
   - Health monitoring

3. **Cloud Deployment**
   - AWS integration
   - Google Cloud support
   - Infrastructure as Code

## Security Considerations

1. **Data Security**
   - Encryption at rest
   - Secure communication
   - Key management

2. **Access Control**
   - Role-based permissions
   - Resource isolation
   - Audit logging

3. **Compliance**
   - GDPR compliance
   - SOC2 requirements
   - Security best practices

## Performance Optimizations

1. **Database**
   - Connection pooling
   - Query optimization
   - Indexing strategy

2. **Caching**
   - Redis caching
   - In-memory caches
   - Cache invalidation

3. **Resource Management**
   - Efficient GPU allocation
   - Memory management
   - Connection pooling

## Development Guidelines

1. **Code Style**
   - Follow Rust style guide
   - Use clippy for linting
   - Document public APIs

2. **Git Workflow**
   - Feature branch workflow
   - Pull request reviews
   - Semantic versioning

3. **Documentation**
   - Keep API docs updated
   - Document breaking changes
   - Maintain changelog

## Monitoring and Logging

1. **Metrics**
   - Request latency
   - Error rates
   - Resource utilization

2. **Logging**
   - Structured logging
   - Log aggregation
   - Error tracking

3. **Alerting**
   - Performance alerts
   - Error thresholds
   - Resource monitoring

## Future Improvements

1. **Technical Debt**
   - Code optimization opportunities
   - Deprecated features
   - Upgrade paths

2. **Feature Roadmap**
   - Planned enhancements
   - Scaling improvements
   - New integrations

3. **Architecture Evolution**
   - Scalability improvements
   - Performance optimizations
   - Security enhancements
