# Contributing to LotaBots

## Getting Started

### Prerequisites

1. Install Rust (1.70 or later)
2. Install PostgreSQL (13 or later)
3. Install NVIDIA drivers (for hardware attestation)
4. Install development tools:
   ```bash
   ./scripts/setup-dev.sh
   ```

### Development Environment Setup

1. Fork and clone the repository
2. Set up environment:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```
3. Build the project:
   ```bash
   cargo build
   ```

## Development Workflow

### 1. Branching

- Create feature branches from `main`
- Use descriptive names: `feature/`, `fix/`, `docs/`, etc.
- Keep branches focused and small

### 2. Code Style

- Follow Rust style guidelines
- Use `cargo fmt` before committing
- Run `cargo clippy` for linting
- Document public APIs

### 3. Commit Messages

Format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:

- feat: New feature
- fix: Bug fix
- docs: Documentation
- style: Formatting
- refactor: Code restructuring
- test: Adding tests
- chore: Maintenance

Example:

```
feat(hardware): add GPU capability detection

Implement NVIDIA GPU capability detection using NVML.
- Add structs for GPU info
- Implement detection logic
- Add error handling

Closes #123
```

### 4. Testing

Required tests:

- Unit tests for business logic
- Integration tests for APIs
- Security tests for critical features
- Performance tests where relevant

Running tests:

```bash
# Run all tests
cargo test

# Run specific tests
cargo test -p hardware-attestation
cargo test -p ai-attestation

# Run with logging
RUST_LOG=debug cargo test
```

### 5. Documentation

Required documentation:

- Public API documentation
- README updates
- Architecture changes
- Security implications

### 6. Pull Requests

Checklist:

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Code formatted
- [ ] Clippy checks pass
- [ ] Security considerations addressed

## Code Review Process

### Reviewer Guidelines

1. Code Quality

   - Clean and maintainable
   - Follows best practices
   - Properly documented

2. Security

   - No obvious vulnerabilities
   - Proper error handling
   - Input validation

3. Performance

   - Efficient algorithms
   - Resource usage
   - Scalability concerns

4. Testing
   - Adequate test coverage
   - Edge cases handled
   - Security tests

### Author Guidelines

1. Keep PRs focused
2. Respond to feedback promptly
3. Update based on reviews
4. Resolve conflicts

## Release Process

### Version Numbering

Follow Semantic Versioning:

- MAJOR: Breaking changes
- MINOR: New features
- PATCH: Bug fixes

### Release Checklist

1. Update version numbers
2. Update CHANGELOG.md
3. Run full test suite
4. Create release branch
5. Create release tag
6. Update documentation

## Getting Help

### Resources

- [Architecture Documentation](ARCHITECTURE.md)
- [Security Guidelines](SECURITY.md)
- [API Documentation](docs/api/README.md)

### Contacts

- Technical Questions: tech@lotabots.com
- Security Issues: security@lotabots.com
- General Help: support@lotabots.com

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Focus on constructive feedback
- Maintain professional discourse
- Support fellow contributors

### Enforcement

- Report violations to conduct@lotabots.com
- Maintainers will review and act
- Actions range from warning to ban
