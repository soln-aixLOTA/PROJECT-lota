# LotaBots AI Gateway

Enterprise-grade AI agent platform with integrated AIOps and DevOps automation.

## Project Description

LotaBots AI Gateway is designed to streamline and automate various aspects of AI and DevOps workflows. It aims to provide a comprehensive solution for CI/CD pipelines, security scanning, automated deployments, and AIOps monitoring. This cutting-edge platform leverages the latest advancements in AI and machine learning to offer predictive scaling, anomaly detection in logs, and automated alerting mechanisms, ensuring high reliability and efficiency in deployment and operations.

## Features

- **Automated CI/CD pipeline**: Streamline your development process with automated testing and deployment.
- **Comprehensive test coverage**: Ensure your application is thoroughly tested.
- **Security scanning and dependency auditing**: Identify and fix security vulnerabilities and outdated dependencies.
- **Automated deployment with Docker and Kubernetes**: Simplify deployment processes.
- **AIOps monitoring and alerting**: Monitor system health and receive alerts.
- **Predictive scaling**: Automatically scale your application based on usage patterns.
- **Log analysis with anomaly detection**: Detect and analyze anomalies in log data.

## Quickstart

### Using Docker Compose

```yaml
version: '3.8'

services:
  api-gateway:
    build: ./api-gateway
    environment:
      - VAULT_ADDR=http://vault.lotabots.svc:8200
      - VAULT_ROLE=api-gateway
      - VAULT_NAMESPACE=lotabots
    ports:
      - "8080:8080"

  frontend:
    build: ./app
    ports:
      - "3000:3000"
```

To start the services, run:
```bash
docker-compose up
```

### Using Rust’s Cargo

```bash
# Clone the repository
git clone <repository-url>

# Navigate to the project directory
cd PROJECT-lota

# Build and run the project
cargo build
cargo run
```

## Prerequisites and Compatibility

- **Rust**: Version 1.55 or higher
- **Docker**: Version 20.10 or higher
- **Kubernetes**: Version 1.21 or higher
- **Platforms**: Linux, macOS, Windows

## DevOps Workflows

### Continuous Integration (ci.yml)

- **Automated testing with PostgreSQL integration**: Ensure your code works with PostgreSQL.
- **Code formatting checks (rustfmt)**: Maintain consistent code style.
- **Linting (clippy)**: Identify and fix potential issues in your code.
- **Security auditing (cargo audit)**: Check for security vulnerabilities.
- **Dependency review**: Keep your dependencies up to date.

### Continuous Deployment (cd.yml)

- **Docker image building and pushing**: Build and push Docker images automatically.
- **Automated deployment on tag releases**: Deploy your application on tag releases.
- **Container registry integration**: Integrate with container registries.
- **Build caching for faster builds**: Improve build times with caching.

### AIOps Automation (aiops.yml)

- **Service health monitoring**: Monitor the health of your services.
- **Resource usage tracking**: Track resource usage.
- **Log analysis with anomaly detection**: Detect anomalies in log data.
- **Predictive scaling**: Automatically scale your application based on predictions.
- **Automated alerting via Slack**: Receive alerts through Slack.

## Setup

1. **Configure GitHub Secrets**:
   ```bash
   SLACK_BOT_TOKEN - For alerts
   DOCKER_USERNAME - For container registry
   DOCKER_PASSWORD - For container registry
   ```

2. **Install Python dependencies**:
   ```bash
   cd scripts
   pip install -r requirements.txt
   ```

3. **Configure monitoring**:
   - Update API endpoints in `aiops.yml`.
   - Configure Slack channel in `aiops.yml`.
   - Adjust scaling thresholds in `predict_scaling.py`.

## Environment Variables and Secrets

- `VAULT_ADDR`: Vault server address (default: `http://vault.lotabots.svc:8200`)
- `VAULT_ROLE`: Kubernetes authentication role (default: `api-gateway`)
- `VAULT_NAMESPACE`: Vault namespace for secrets (default: `lotabots`)

## Development

1. **Run tests**:
   ```bash
   cargo test
   ```

2. **Check formatting**:
   ```bash
   cargo fmt --all -- --check
   ```

3. **Run linter**:
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Security audit**:
   ```bash
   cargo audit
   ```

## Deployment

1. **Create a new release**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Monitor deployment**:
   - Check GitHub Actions.
   - Verify container registry.
   - Monitor service health.

## AIOps Features

### Log Analysis

- Anomaly detection in log patterns.
- Error clustering and categorization.
- Automated reporting.

### Predictive Scaling

- Traffic pattern analysis.
- Resource usage prediction.
- Automated HPA configuration.

### Monitoring

- Health checks.
- Resource usage tracking.
- Performance metrics.
- Automated alerting.

## Troubleshooting Tips / FAQ

- **Authentication Failures**:
  - Verify the Kubernetes service account token is mounted.
  - Check the Vault role binding is correct.
  - Ensure network policies allow communication with Vault.
- **Secret Access Denied**:
  - Verify the Vault policy is correctly applied.
  - Check the secret path matches the policy.
  - Ensure the service account has the correct role binding.
- **Lease Renewal Failures**:
  - Check Vault server connectivity.
  - Verify the lease TTL is within allowed limits.
  - Ensure the policy allows lease renewal.

## Additional Resources

- [Slack API Integration](https://api.slack.com/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)

## Security Hardening

- Regularly update and re-scan dependencies.
- Follow best practices for Rust security audits.

## Contributions and Code of Conduct

1. Fork the repository.
2. Create a feature branch.
3. Submit a pull request.
4. Refer to the [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for more details.

## License


