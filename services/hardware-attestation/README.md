# Hardware Attestation Service

## Overview

The Hardware Attestation Service is responsible for verifying and attesting hardware configurations in the LotaBots platform. It provides capabilities for hardware verification, cloud storage integration, and secure attestation of system components.

## Features

- Hardware verification using NVML (NVIDIA Management Library)
- Cloud storage integration (AWS S3 and Google Cloud Storage)
- Secure logging and tracing
- UUID-based attestation tracking

## Dependencies

- NVML Wrapper for hardware interaction
- AWS SDK for S3 storage
- Google Cloud Storage for backup storage
- Tokio for async runtime
- Various utility crates for logging, serialization, etc.

## Configuration

The service can be configured through environment variables and TOML configuration files. See the documentation for detailed configuration options.

## Development

```bash
# Build the service
cargo build

# Run tests
cargo test

# Run the service
cargo run
```

## Testing

The service includes comprehensive tests:

- Unit tests
- Integration tests with mock hardware
- Wiremock-based API testing
