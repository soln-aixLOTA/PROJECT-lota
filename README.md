# LotaBots Platform

A secure, scalable, and enterprise-ready AI platform built with Rust and modern cloud-native technologies.

## Project Structure

```
lotabots/
├── services/               # Microservices
│   ├── api-gateway/       # API Gateway & Request Router
│   ├── user-management/   # User & Tenant Management
│   ├── inference-service/ # AI Model Inference
│   └── attestation-service/ # Security Attestation
├── common/                # Shared Libraries
│   ├── middleware/        # Common Middleware
│   ├── auth/             # Authentication
│   └── types/            # Shared Types
├── infrastructure/        # Infrastructure Code
│   ├── k8s/              # Kubernetes Manifests
│   ├── helm/             # Helm Charts
│   └── terraform/        # Terraform Configurations
└── tests/                # Testing
    ├── unit/             # Unit Tests
    ├── integration/      # Integration Tests
    └── e2e/              # End-to-End Tests
```

## Prerequisites

- Rust 1.70+ (with cargo)
- Docker 24.0+
- Kubernetes 1.25+
- GPU Support (NVIDIA drivers & CUDA toolkit)

## Quick Start

1. **Setup Development Environment**

   ```bash
   # Clone the repository
   git clone https://github.com/lotabots/platform.git
   cd platform

   # Install dependencies
   cargo build
   ```

2. **Run Services Locally**

   ```bash
   # Start all services
   docker-compose up

   # Or run individual services
   cd services/api-gateway
   cargo run
   ```

3. **Run Tests**

   ```bash
   # Run all tests
   cargo test

   # Run specific service tests
   cargo test -p api-gateway
   ```

## Service Documentation

### API Gateway

- Main entry point for all API requests
- Handles authentication, rate limiting, and request routing
- [API Gateway Documentation](services/api-gateway/README.md)

### User Management

- Manages users, tenants, and permissions
- Handles authentication and authorization
- [User Management Documentation](services/user-management/README.md)

### Inference Service

- Handles AI model inference requests
- GPU resource management and optimization
- [Inference Service Documentation](services/inference-service/README.md)

### Attestation Service

- Provides security attestation for AI models
- Manages model verification and validation
- [Attestation Service Documentation](services/attestation-service/README.md)

## Development Guidelines

### Code Style

- Follow Rust style guidelines
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting

### Git Workflow

1. Create feature branch from `main`
2. Make changes and commit
3. Run tests locally
4. Create pull request
5. Wait for CI checks and review
6. Merge to `main`

### Testing Requirements

- Unit tests for all new code
- Integration tests for API endpoints
- E2E tests for critical flows
- Minimum 80% test coverage

## Deployment

### Local Development

```bash
docker-compose up
```

### Kubernetes

```bash
# Deploy using Helm
cd infrastructure/helm
helm install lotabots ./lotabots-chart

# Or using kubectl
kubectl apply -f infrastructure/k8s/
```

### Production Deployment

See [Deployment Guide](docs/deployment.md) for detailed instructions.

## Monitoring & Observability

- Prometheus metrics exposed on `/metrics`
- Grafana dashboards in `infrastructure/monitoring`
- Distributed tracing with Jaeger
- Structured logging with JSON format

## Security

- All endpoints require authentication
- JWT-based authorization
- Rate limiting enabled
- Regular security audits
- See [Security Policy](SECURITY.md)

## Contributing

1. Read [Contributing Guidelines](CONTRIBUTING.md)
2. Fork the repository
3. Create feature branch
4. Make changes
5. Create pull request

## License

[License details here]

## Support

- [Issue Tracker](https://github.com/lotabots/platform/issues)
- [Documentation](docs/)
