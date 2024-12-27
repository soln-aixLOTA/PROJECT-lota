# LotaBots AI Gateway

Enterprise-grade AI agent platform with integrated AIOps and DevOps automation.

## Features

- Automated CI/CD pipeline
- Comprehensive test coverage
- Security scanning and dependency auditing
- Automated deployment with Docker and Kubernetes
- AIOps monitoring and alerting
- Predictive scaling
- Log analysis with anomaly detection

## DevOps Workflows

### Continuous Integration (ci.yml)

- Automated testing with PostgreSQL integration
- Code formatting checks (rustfmt)
- Linting (clippy)
- Security auditing (cargo audit)
- Dependency review

### Continuous Deployment (cd.yml)

- Docker image building and pushing
- Automated deployment on tag releases
- Container registry integration
- Build caching for faster builds

### AIOps Automation (aiops.yml)

- Service health monitoring
- Resource usage tracking
- Log analysis with anomaly detection
- Predictive scaling
- Automated alerting via Slack

## Setup

1. Configure GitHub Secrets:
   ```bash
   SLACK_BOT_TOKEN - For alerts
   DOCKER_USERNAME - For container registry
   DOCKER_PASSWORD - For container registry
   ```

2. Install Python dependencies:
   ```bash
   cd scripts
   pip install -r requirements.txt
   ```

3. Configure monitoring:
   - Update API endpoints in aiops.yml
   - Configure Slack channel in aiops.yml
   - Adjust scaling thresholds in predict_scaling.py

## Development

1. Run tests:
   ```bash
   cargo test
   ```

2. Check formatting:
   ```bash
   cargo fmt --all -- --check
   ```

3. Run linter:
   ```bash
   cargo clippy -- -D warnings
   ```

4. Security audit:
   ```bash
   cargo audit
   ```

## Deployment

1. Create a new release:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. Monitor deployment:
   - Check GitHub Actions
   - Verify container registry
   - Monitor service health

## AIOps Features

### Log Analysis
- Anomaly detection in log patterns
- Error clustering and categorization
- Automated reporting

### Predictive Scaling
- Traffic pattern analysis
- Resource usage prediction
- Automated HPA configuration

### Monitoring
- Health checks
- Resource usage tracking
- Performance metrics
- Automated alerting

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License - see LICENSE file for details
