# Git Hooks Documentation

This document describes the Git hooks used in the LotaBots project to maintain code quality, security, and consistency across the codebase.

## Table of Contents
- [Overview](#overview)
- [Pre-commit Hook](#pre-commit-hook)
- [Commit Message Hook](#commit-message-hook)
- [Installation](#installation)
- [Bypassing Hooks](#bypassing-hooks)
- [Troubleshooting](#troubleshooting)

## Overview

The LotaBots project uses two primary Git hooks:
1. **pre-commit**: Runs before creating a commit to ensure code quality and security
2. **commit-msg**: Validates commit messages to maintain consistent documentation

## Pre-commit Hook

The pre-commit hook performs the following checks:

### 1. Rust Formatting & Linting
- Runs `cargo fmt` to ensure consistent code style
- Executes `cargo clippy` to catch common mistakes and enforce best practices

### 2. Security Checks
Scans for sensitive data patterns including:

#### API and Authentication
- API keys and secrets
- JWT tokens
- Authentication tokens
- OAuth credentials

#### Cloud Provider Credentials
- AWS (Secret keys, Access keys, Session tokens)
- Azure (Storage keys, Connection strings, AD secrets)
- GCP (API keys, Client secrets, Service account credentials)

#### Database and Cache
- Database URLs and credentials
- Redis connection strings
- PostgreSQL/MongoDB credentials

#### AI/ML Related
- OpenAI API keys
- HuggingFace tokens
- Model access keys
- Anthropic/Cohere keys

### 3. Microservice-Specific Checks
- Validates Cargo.toml version consistency
- Checks for proper error handling:
  - unwrap()/expect() usage
  - panic! macro usage
  - todo! implementations
  - dbg! macro removal

### 4. Configuration and Protocol Checks
- Validates .proto files:
  - Package declarations
  - Syntax version
  - Language-specific options
- Checks configuration files:
  - TOML syntax and required fields
  - YAML/K8s manifest validation
  - JSON structure and required fields

### 5. Documentation Checks
- Ensures public items are documented
- Flags TODO/FIXME comments
- Verifies documentation standards

### 6. Test Coverage
- Runs all tests
- Executes service-specific tests
- Verifies test file existence

## Commit Message Hook

The commit-msg hook enforces the [Conventional Commits](https://www.conventionalcommits.org/) specification with LotaBots-specific extensions.

### Format
```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types
1. **Standard Types**
   - feat: New features
   - fix: Bug fixes
   - docs: Documentation changes
   - style: Formatting changes
   - refactor: Code refactoring
   - perf: Performance improvements
   - test: Adding/updating tests
   - build: Build system changes
   - ci: CI configuration changes
   - chore: Maintenance tasks
   - revert: Reverting changes

2. **LotaBots-Specific Types**
   - attestation: Hardware/AI attestation changes
   - document: Document processing features
   - resource: Resource management updates
   - auth: Authentication/authorization changes
   - api: API-related modifications
   - agent: Agent system changes
   - ml: Machine learning components
   - data: Data processing features
   - ui: User interface changes
   - infra: Infrastructure updates
   - hardware: Hardware integration
   - security: Security enhancements
   - monitoring: Monitoring/observability
   - config: Configuration changes
   - db: Database modifications
   - cache: Caching system updates
   - queue: Queue system changes
   - metrics: Metrics/telemetry updates

### Scopes
1. **Core Services**
   - auth, api, agent, attestation, document, resource

2. **Infrastructure**
   - aws, azure, gcp, k8s, docker

3. **Data & ML**
   - ml-pipeline, training, inference, dataset

4. **Monitoring & Observability**
   - metrics, traces, logs, alerts

5. **Storage & Caching**
   - postgres, redis, s3, cache

6. **Security & Compliance**
   - audit, compliance, encryption, secrets

### Examples
```bash
feat(auth): implement JWT token refresh
fix(attestation): resolve hardware verification timeout
perf(ml-pipeline): optimize model inference
security(encryption): update TLS configuration
```

## Installation

The hooks are automatically installed in the `.git/hooks` directory. To manually install or update:

```bash
# Make hooks executable
chmod +x .git/hooks/pre-commit
chmod +x .git/hooks/commit-msg
```

## Bypassing Hooks

In rare cases, you may need to bypass the hooks:

```bash
# Bypass pre-commit hook
git commit --no-verify -m "your message"

# Bypass both hooks
git commit --no-verify --allow-empty-message -m "your message"
```

⚠️ **Warning**: Only bypass hooks when absolutely necessary and you understand the implications.

## Troubleshooting

### Common Issues

1. **Security Check Failures**
   - Check for hardcoded credentials
   - Use environment variables or secure vaults
   - Remove debug/test credentials

2. **Commit Message Format**
   - Ensure type and scope are valid
   - Use imperative mood in descriptions
   - Keep lines under 100 characters
   - Reference issue numbers (e.g., LOTA-123)

3. **Configuration Validation**
   - Verify YAML/JSON syntax
   - Include required fields
   - Follow language-specific conventions

4. **Test Failures**
   - Run tests locally before committing
   - Check service-specific test suites
   - Ensure test files exist for new code

### Getting Help

For additional assistance:
1. Check the error messages for specific guidance
2. Review this documentation
3. Contact the development team
4. File an issue in the repository
