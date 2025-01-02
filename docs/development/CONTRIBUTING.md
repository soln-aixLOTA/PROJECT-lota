# Contributing to LotaBots

We welcome contributions to the LotaBots platform! This document provides guidelines and best practices for contributing.

## Development Process

### 1. Setting Up Development Environment
- Fork the repository
- Clone your fork
- Install prerequisites (see README.md)
- Set up pre-commit hooks

### 2. Code Standards
- **Rust Code**: Follow Rust 2021 Edition guidelines
- **Python Code**: Follow PEP 8
- **JavaScript/TypeScript**: Follow Airbnb style guide
- Use meaningful variable/function names
- Document all public APIs
- Write comprehensive tests

### 3. Making Changes
1. Create a feature branch
2. Write tests first (TDD approach)
3. Implement changes
4. Update documentation
5. Run full test suite
6. Submit PR

### 4. Pull Request Process
1. Update relevant documentation
2. Add tests for new features
3. Ensure CI passes
4. Get review from at least two team members
5. Squash commits before merging

## Testing Guidelines
- Write unit tests for all new code
- Include integration tests for API changes
- E2E tests for user-facing features
- Maintain >90% code coverage

## Documentation
- Update API documentation
- Include inline code comments
- Update relevant architecture docs
- Add examples for new features

## Security
- Follow security guidelines in SECURITY.md
- Never commit secrets or credentials
- Report security issues privately

## Questions?
- Open a discussion
- Contact maintainers
- Check existing documentation 