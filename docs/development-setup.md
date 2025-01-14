# Development Setup Guide

This document provides comprehensive setup instructions and coding guidelines for developers contributing to the LotaBots project. Follow these guidelines to ensure consistent development practices across the project. For a high-level overview of the project, see the [main README](../README.md).

## General Setup

Before starting development, ensure you have the following tools installed:

- Git for version control
- Docker for containerization (see [deployment guide](./deployment/README.md))
- Your preferred IDE/editor with support for:
  - Code formatting
  - Linting
  - Language servers
  - Git integration

For security requirements and guidelines, refer to our [security documentation](./security.md).

## Language-Specific Setup

### Rust Setup

1. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install Required Tools:**
   ```bash
   # Install formatting tool
   rustup component add rustfmt

   # Install linting tool
   rustup component add clippy
   ```

3. **Configure Your Editor:**
   - Install Rust Analyzer extension
   - Enable format on save
   - Enable clippy linting

4. **Coding Standards:**
   - Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/README.html)
   - Use `cargo fmt` before committing changes
   - Run `cargo clippy` to catch common mistakes
   - Document public APIs using `///` doc comments
   - Handle errors explicitly using `Result`
   - Avoid `panic!` in library code

For detailed API documentation, see [API Guide](./api.md).

### Python Setup

1. **Install Python:**
   ```bash
   # Install Python 3.8 or higher
   sudo apt-get update
   sudo apt-get install python3.8 python3.8-venv
   ```

2. **Set Up Virtual Environment:**
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate
   ```

3. **Install Development Tools:**
   ```bash
   pip install flake8 pylint black mypy
   ```

4. **Configure Your Editor:**
   - Install Python extension
   - Enable format on save using Black
   - Configure linting with flake8 and pylint

5. **Coding Standards:**
   - Follow [PEP 8](https://peps.python.org/pep-0008/)
   - Use flake8 and pylint for code quality
   - Write docstrings for functions and classes
   - Use type hints for better code clarity

### JavaScript/TypeScript Setup

1. **Install Node.js:**
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   ```

2. **Install Development Tools:**
   ```bash
   npm install -g eslint prettier typescript
   ```

3. **Project Setup:**
   ```bash
   # Initialize a new project
   npm init

   # Install development dependencies
   npm install --save-dev eslint prettier @typescript-eslint/parser @typescript-eslint/eslint-plugin
   ```

4. **Configure Your Editor:**
   - Install ESLint extension
   - Install Prettier extension
   - Enable format on save
   - Set up TypeScript language server

5. **Coding Standards:**
   - Follow the [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
   - Use ESLint for code quality
   - Write JSDoc comments for functions and classes
   - Use TypeScript for type safety

### Go Setup

1. **Install Go:**
   ```bash
   # Download and install Go
   wget https://go.dev/dl/go1.21.0.linux-amd64.tar.gz
   sudo tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz
   export PATH=$PATH:/usr/local/go/bin
   ```

2. **Configure Your Editor:**
   - Install Go extension
   - Enable format on save
   - Set up Go language server

3. **Coding Standards:**
   - Follow conventions established by `gofmt`
   - Use `go vet` for static analysis
   - Document public APIs
   - Handle errors explicitly

## Version Control Guidelines

1. **Git Configuration:**
   ```bash
   git config --global user.name "Your Name"
   git config --global user.email "your.email@example.com"
   ```

2. **Branching Strategy:**
   - Use Gitflow workflow
   - Branch naming convention:
     - Feature: `feature/description`
     - Bugfix: `bugfix/description`
     - Release: `release/version`

3. **Commit Guidelines:**
   - Write meaningful commit messages
   - Use present tense ("Add feature" not "Added feature")
   - Reference issue numbers when applicable

For security-related commits, refer to our [security guidelines](./security.md#security-policies).

## Code Review Process

1. **Before Submitting:**
   - Run all tests
   - Apply code formatting
   - Run linting tools
   - Update documentation if needed
   - Follow [security best practices](./security.md#security-policies)

2. **Pull Request Guidelines:**
   - Provide clear description of changes
   - Include test results
   - Link related issues
   - Keep changes focused and atomic

For detailed API documentation and integration guidelines, see [API Documentation](./api.md).

## Getting Help

If you encounter setup issues:
1. Check the troubleshooting section in [README](../README.md#troubleshooting)
2. Review language-specific documentation
3. Ask in the development chat
4. Create an issue if it's a recurring problem

## Additional Resources

- [Project README](../README.md)
- [Coding Standards](./coding-standards.md)
- [Error Handling Guide](./error-handling.md)
- [Security Guidelines](./security.md)
- [API Documentation](./api.md)
- [Documentation Map](./documentation-map.md)
