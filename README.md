# LotaBots Platform

## Overview

LotaBots is a comprehensive platform for managing and attesting AI and hardware components. The platform consists of multiple services working together to provide secure, reliable, and scalable AI operations.

## Project Structure

```
services/
├── hardware-attestation/  # Hardware verification and attestation service
└── ai-attestation/       # AI model verification and compliance service
```

## Services

### Hardware Attestation Service

Responsible for verifying and attesting hardware configurations:

- Hardware verification using NVML
- Cloud storage integration
- Secure logging and tracing
  See [hardware-attestation/README.md](services/hardware-attestation/README.md) for details.

### AI Attestation Service

Web service for AI model verification and compliance:

- RESTful API endpoints
- PostgreSQL database integration
- OpenTelemetry observability
  See [ai-attestation/README.md](services/ai-attestation/README.md) for details.

## Development

### Prerequisites

- Rust 1.70 or later
- PostgreSQL 13 or later
- NVIDIA drivers (for hardware attestation)

### Building

```bash
# Build all services
cargo build

# Build specific service
cargo build -p hardware-attestation
cargo build -p ai-attestation
```

### Testing

```bash
# Run all tests
cargo test

# Test specific service
cargo test -p hardware-attestation
cargo test -p ai-attestation
```

## Configuration

Each service has its own configuration requirements. See the respective service README files for details.

## Contributing

1. Ensure you have the latest Rust toolchain
2. Fork the repository
3. Create a feature branch
4. Submit a pull request

## License

[Add appropriate license information]
