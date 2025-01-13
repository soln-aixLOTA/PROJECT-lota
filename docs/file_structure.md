# LotaBots File Structure

## Overview

This document provides a comprehensive overview of the LotaBots platform's file structure, detailing the purpose and contents of each directory and key file.

## Root Directory Structure

```
lotabots/
├── services/           # Microservices
├── shared/            # Shared libraries and utilities
├── infrastructure/    # Infrastructure as code
├── scripts/          # Development and deployment scripts
├── docs/             # Documentation
├── tests/            # Integration and end-to-end tests
└── tools/            # Development tools and utilities
```

## Services Directory

### API Gateway Service
```
services/api-gateway/
├── src/
│   ├── main.rs                 # Service entry point
│   ├── config.rs               # Configuration handling
│   ├── routes/
│   │   ├── mod.rs             # Routes module
│   │   ├── auth.rs            # Authentication routes
│   │   ├── documents.rs       # Document management routes
│   │   └── attestation.rs     # Attestation routes
│   ├── middleware/
│   │   ├── mod.rs             # Middleware module
│   │   ├── auth.rs            # Authentication middleware
│   │   ├── rate_limit.rs      # Rate limiting
│   │   └── logging.rs         # Request logging
│   └── error.rs               # Error handling
├── Cargo.toml                  # Dependencies and metadata
└── Dockerfile                  # Container configuration
```

### Authentication Service
```
services/auth/
├── src/
│   ├── main.rs                # Service entry point
│   ├── config.rs              # Configuration handling
│   ├── models/
│   │   ├── mod.rs            # Models module
│   │   ├── user.rs           # User model
│   │   └── session.rs        # Session model
│   ├── handlers/
│   │   ├── mod.rs            # Handlers module
│   │   ├── login.rs          # Login handler
│   │   ├── register.rs       # Registration handler
│   │   └── token.rs          # Token management
│   ├── repository/
│   │   ├── mod.rs            # Repository module
│   │   └── postgres.rs       # PostgreSQL implementation
│   └── error.rs              # Error handling
├── migrations/                # Database migrations
├── Cargo.toml                # Dependencies and metadata
└── Dockerfile               # Container configuration
```

### Hardware Attestation Service
```
services/attestation/
├── src/
│   ├── main.rs               # Service entry point
│   ├── config.rs             # Configuration handling
│   ├── models/
│   │   ├── mod.rs           # Models module
│   │   ├── hardware.rs      # Hardware model
│   │   └── verification.rs  # Verification model
│   ├── handlers/
│   │   ├── mod.rs           # Handlers module
│   │   ├── submit.rs        # Submission handler
│   │   └── verify.rs        # Verification handler
│   ├── verifier/
│   │   ├── mod.rs           # Verifier module
│   │   └── hardware.rs      # Hardware verification logic
│   └── error.rs             # Error handling
├── Cargo.toml               # Dependencies and metadata
└── Dockerfile              # Container configuration
```

### Document Service
```
services/document/
├── src/
│   ├── main.rs              # Service entry point
│   ├── config.rs            # Configuration handling
│   ├── models/
│   │   ├── mod.rs          # Models module
│   │   └── document.rs     # Document model
│   ├── handlers/
│   │   ├── mod.rs          # Handlers module
│   │   ├── upload.rs       # Upload handler
│   │   └── download.rs     # Download handler
│   ├── storage/
│   │   ├── mod.rs          # Storage module
│   │   └── s3.rs           # S3 implementation
│   └── error.rs            # Error handling
├── Cargo.toml              # Dependencies and metadata
└── Dockerfile             # Container configuration
```

### Resource Management Service
```
services/resource-management/
├── src/
│   ├── main.rs             # Service entry point
│   ├── config.rs           # Configuration handling
│   ├── models/
│   │   ├── mod.rs         # Models module
│   │   ├── resource.rs    # Resource model
│   │   └── allocation.rs  # Allocation model
│   ├── handlers/
│   │   ├── mod.rs         # Handlers module
│   │   ├── allocate.rs    # Allocation handler
│   │   └── monitor.rs     # Resource monitoring
│   ├── scheduler/
│   │   ├── mod.rs         # Scheduler module
│   │   └── kubernetes.rs  # Kubernetes integration
│   └── error.rs           # Error handling
├── Cargo.toml             # Dependencies and metadata
└── Dockerfile            # Container configuration
```

## Shared Libraries

### Database Library
```
shared/db/
├── src/
│   ├── lib.rs            # Library entry point
│   ├── config.rs         # Database configuration
│   ├── connection.rs     # Connection management
│   ├── migrations.rs     # Migration handling
│   └── models/
│       ├── mod.rs        # Models module
│       ├── user.rs       # User model
│       └── common.rs     # Common types
└── Cargo.toml           # Dependencies and metadata
```

### Utils Library
```
shared/utils/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── logging.rs       # Logging utilities
│   ├── metrics.rs       # Metrics collection
│   ├── tracing.rs       # Distributed tracing
│   └── error.rs         # Error types
└── Cargo.toml          # Dependencies and metadata
```

## Infrastructure

### Terraform
```
infrastructure/terraform/
├── main.tf             # Main Terraform configuration
├── variables.tf        # Input variables
├── outputs.tf         # Output values
├── vpc.tf            # VPC configuration
├── eks.tf           # EKS cluster configuration
├── rds.tf          # RDS configuration
└── modules/        # Custom Terraform modules
```

### Kubernetes
```
infrastructure/kubernetes/
├── base/
│   ├── namespace.yaml          # Namespace definition
│   ├── serviceaccount.yaml     # Service account
│   └── rbac.yaml              # RBAC configuration
├── services/
│   ├── api-gateway/           # API Gateway manifests
│   ├── auth/                  # Auth service manifests
│   ├── attestation/          # Attestation service manifests
│   ├── document/            # Document service manifests
│   └── resource-management/ # Resource management manifests
└── monitoring/
    ├── prometheus/          # Prometheus configuration
    ├── grafana/            # Grafana configuration
    └── jaeger/            # Jaeger configuration
```

## Scripts
```
scripts/
├── development/
│   ├── setup.sh           # Development environment setup
│   └── test.sh           # Run test suites
├── deployment/
│   ├── deploy.sh         # Deployment script
│   └── rollback.sh       # Rollback script
└── monitoring/
    ├── metrics.sh        # Metrics collection
    └── alerts.sh         # Alert configuration
```

## Documentation
```
docs/
├── architecture/
│   ├── overview.md          # System architecture
│   ├── data_flow.md        # Data flow diagrams
│   └── decisions.md        # Architecture decisions
├── deployment/
│   ├── guide.md            # Deployment guide
│   └── configuration.md    # Configuration guide
├── development/
│   ├── setup.md           # Development setup
│   └── guidelines.md      # Coding guidelines
├── security/
│   ├── overview.md        # Security overview
│   └── compliance.md      # Compliance documentation
└── api/
    └── openapi.yaml       # API specification
```

## Tests
```
tests/
├── integration/
│   ├── api_gateway/      # API Gateway tests
│   ├── auth/            # Auth service tests
│   ├── attestation/    # Attestation service tests
│   └── document/      # Document service tests
└── e2e/
    ├── scenarios/     # Test scenarios
    └── fixtures/     # Test data
```

## Tools
```
tools/
├── ci/
│   ├── lint.sh         # Linting tools
│   └── security.sh    # Security scanning
├── metrics/
│   └── dashboard.sh   # Dashboard setup
└── development/
    └── ide/          # IDE configurations
```
