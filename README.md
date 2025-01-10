# Document Automation Service Scripts

This directory contains scripts for managing and testing the Document Automation Service.

## Directory Structure

```
.
├── config/             # Environment-specific configurations
│   ├── dev/           # Development environment
│   ├── test/          # Test environment
│   └── prod/          # Production environment
├── lib/               # Common library functions
├── tests/             # Test scripts and resources
└── scripts/           # Main scripts
```

## Scripts

- `set-env.sh`: Set up environment-specific configuration
- `dev-token.sh`: Generate JWT tokens for development and testing
- `load-test.sh`: Run load tests against the service
- `test-all.sh`: Run all test scenarios
- `db-manage.sh`: Database management utilities
- `setup-dev.sh`: Set up development environment

## Environment Configuration

Each environment (dev, test, prod) has its own configuration file in the `config/` directory. Use `set-env.sh` to switch between environments:

```bash
# Set development environment
./set-env.sh --env dev

# Set test environment
./set-env.sh --env test

# Set production environment
./set-env.sh --env prod
```

## Running Tests

To run all tests:

```bash
# Run all tests in test environment
./test-all.sh

# Run specific test scenario
./load-test.sh --scenario crud --token "your-token"
```

Available test scenarios:

- `crud`: Basic CRUD operations
- `workflow`: Document workflow operations
- `mixed`: Mixed operations
- `security`: Security-related tests

## Development

To set up the development environment:

```bash
# Set up development environment
./setup-dev.sh

# Generate development token
./dev-token.sh --user test --role admin
```

## Configuration

Environment-specific configuration files (`config/<env>/config.sh`) contain:

- Service URLs and endpoints
- Database connection details
- JWT configuration
- Test parameters
- Monitoring endpoints

## Common Library

The `lib/common.sh` file contains shared functions for:

- Logging and output formatting
- Environment validation
- Docker and k6 checks
- Test result validation

## Contributing

1. Use the common library functions for consistency
2. Follow the script organization structure
3. Update environment configurations as needed
4. Test changes in development environment first
