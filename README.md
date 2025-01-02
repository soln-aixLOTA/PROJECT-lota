# LotaBots AI Gateway

Enterprise-grade AI agent platform with integrated AIOps and DevOps automation.

## Table of Contents

- [LotaBots AI Gateway](#lotabots-ai-gateway)
  - [Table of Contents](#table-of-contents)
  - [Project Description](#project-description)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Deployment](#deployment)
  - [Development](#development)
  - [Monitoring](#monitoring)
  - [Troubleshooting](#troubleshooting)
  - [Contributing](#contributing)
  - [License](#license)

## Project Description

LotaBots AI Gateway is an enterprise-grade AI agent platform that provides secure, scalable, and reliable AI services. It includes built-in AIOps capabilities and DevOps automation for seamless deployment and operation.

## Features

- Secure attestation service for AI agent verification
- API Gateway with rate limiting and authentication
- Kubernetes-native deployment
- Integrated monitoring and tracing
- Automated scaling and failover
- Database migrations and management
- Comprehensive logging and metrics

## Prerequisites

- Rust 1.55 or later
- Docker 20.10 or later
- Kubernetes 1.21 or later
- PostgreSQL 13 or later
- Redis 6 or later
- Vault for secrets management
- GitHub account with permissions to create packages

## Initial Setup

### 1. GitHub Container Registry Setup

1. Create a Personal Access Token (PAT) with `read:packages` and `write:packages` scopes:

   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Click "Generate new token"
   - Select the required scopes
   - Copy the generated token

2. Configure GitHub Actions secrets:

   - Go to your repository settings
   - Navigate to "Secrets and variables" → "Actions"
   - Add the following secrets:
     - `KUBECONFIG`: Your Kubernetes cluster configuration
     - `GITHUB_TOKEN`: Will be automatically available
     - `DOCKER_USERNAME`: Your GitHub username
     - `DOCKER_PASSWORD`: Your GitHub PAT from step 1

3. Enable GitHub Container Registry:
   - Go to your repository settings
   - Navigate to "Packages"
   - Ensure the Container registry is enabled

## Installation

1. Clone the repository:

```bash
git clone https://github.com/your-org/lotabots.git
cd lotabots
```

2. Run the automated setup script:

```bash
chmod +x setup.sh
sudo ./setup.sh
```

This will:

- Install all required dependencies
- Configure the environment
- Set up the database
- Prepare for deployment

## Configuration

The platform can be configured through:

1. Environment variables (see `.env` file)
2. Kubernetes ConfigMaps and Secrets
3. Vault for sensitive information

Key configuration files:

- `.env` - Local development configuration
- `k8s/common/configmap.yaml` - Shared Kubernetes configuration
- `k8s/*/deployment.yaml` - Service-specific configuration

## Deployment

### Local Development

1. Start the services locally:

```bash
cargo run
```

### Kubernetes Deployment

1. Configure your Kubernetes cluster:

```bash
kubectl config use-context your-cluster
```

2. Deploy using the provided script:

```bash
./scripts/deploy.sh
```

The deployment script will:

- Build and push Docker images
- Apply Kubernetes manifests
- Verify the deployment
- Set up monitoring

### Manual Deployment Steps

1. Build the Docker images:

```bash
docker build -t ghcr.io/lotabots/attestation:latest -f src/attestation/Dockerfile .
docker build -t ghcr.io/lotabots/api_gateway:latest -f src/api_gateway/Dockerfile .
```

2. Push the images:

```bash
docker push ghcr.io/lotabots/attestation:latest
docker push ghcr.io/lotabots/api_gateway:latest
```

3. Apply Kubernetes manifests:

```bash
kubectl apply -f k8s/common/
kubectl apply -f k8s/attestation/
kubectl apply -f k8s/api_gateway/
```

## Development

1. Install development dependencies:

```bash
cargo install sqlx-cli
cargo install cargo-watch
```

2. Run tests:

```bash
cargo test
```

3. Start development server:

```bash
cargo watch -x run
```

## Monitoring

The platform includes:

- OpenTelemetry integration
- Prometheus metrics
- Kubernetes health probes
- Structured logging

Access monitoring:

1. Metrics: `http://api-gateway/metrics`
2. Health: `http://api-gateway/health`
3. Kubernetes dashboard for pod status

## Troubleshooting

Common issues and solutions:

1. Database connection issues:

   - Check PostgreSQL is running
   - Verify connection string in `.env`
   - Ensure database migrations are applied

2. Kubernetes deployment issues:
   - Check pod logs: `kubectl logs -f -l app=lotabots`
   - Verify secrets: `kubectl get secrets -n lotabots`
   - Check service status: `kubectl get services -n lotabots`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
