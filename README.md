# Document Automation with vLLM Integration

This project combines a Rust-based document automation service with vLLM for efficient large language model inference.

## Prerequisites

- NVIDIA GPU with CUDA support
- NVIDIA drivers installed
- Docker and docker-compose
- NVIDIA Container Toolkit (nvidia-docker2)
- Hugging Face account and access token

## Environment Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd document-automation
```

2. Create a `.env` file in the project root:
```bash
# Database
DATABASE_URL=postgresql://postgres:postgres@db:5432/document_automation
TEST_DATABASE_URL=postgresql://postgres:postgres@db:5432/document_automation_test

# JWT
JWT_SECRET=your-secret-key
JWT_EXPIRY=15m

# vLLM
HUGGING_FACE_HUB_TOKEN=your-token-here
MODEL_NAME=meta-llama/Llama-2-7b-chat-hf
TENSOR_PARALLEL_SIZE=1
GPU_MEMORY_UTILIZATION=0.90
```

## Development Setup

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install development dependencies:
```bash
cargo install sqlx-cli
```

3. Set up the database:
```bash
cargo sqlx database create
cargo sqlx migrate run
```

## Deployment

1. Make the deployment script executable:
```bash
chmod +x scripts/deploy.sh
```

2. Run the deployment:
```bash
./scripts/deploy.sh
```

The script will:
- Check for NVIDIA prerequisites
- Build and start the Docker containers
- Run database migrations
- Verify service health

## Services

After deployment, the following services will be available:

- Rust Application: <http://localhost:3000>
  - API documentation: <http://localhost:3000/swagger-ui/>
  - Health check: <http://localhost:3000/health>

- vLLM Service: <http://localhost:8000>
  - Model information: <http://localhost:8000/v1/models>
  - OpenAI-compatible endpoint: <http://localhost:8000/v1/completions>

## Testing

Run the test suite:
```bash
cargo test
```

For specific test files:
```bash
cargo test --test auth_jwt_test -- --nocapture
```

## API Examples

1. Register a new user:
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123456",
    "role": "User"
  }'
```

2. Login:
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123456"
  }'
```

3. Create a document (requires authentication):
```bash
curl -X POST http://localhost:3000/documents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-token>" \
  -d '{
    "title": "Test Document",
    "content": "This is a test document."
  }'
```

## License

[Add your license information here]

# LOTA AI

## Database Connection Testing

The application includes a `db_check` binary that can be used to verify database connectivity. This is particularly useful when:
- Setting up the application for the first time
- Troubleshooting connection issues
- Verifying AlloyDB proxy configuration

### Using db_check

1. In development (using Docker Compose):
```bash
docker-compose exec app db_check
```

2. In production:
```bash
db_check
```

The tool will attempt to connect to the database using the configured `DATABASE_URL` environment variable and will:
- Print a success message if the connection is successful
- Print a detailed error message if the connection fails

## Environment Variables

The application uses the following environment variables:

### Database Configuration
- `DATABASE_URL`: PostgreSQL connection string
- `POSTGRES_USER`: Database user (local development)
- `POSTGRES_PASSWORD`: Database password (local development)
- `POSTGRES_DB`: Database name (local development)

### Application Settings
- `PORT`: Server port (default: 3000)
- `RUST_LOG`: Logging level (e.g., debug, info)
- `JWT_SECRET`: Secret key for JWT token generation
- `JWT_EXPIRY`: JWT token expiry time

### Production Settings
- `ALLOYDB_INSTANCE_URI`: AlloyDB instance URI for production
- `ENVIRONMENT`: Application environment (development/production)

## Security Best Practices

1. Credentials Management:
   - Use environment variables for sensitive configuration
   - In production, use Google Cloud Secret Manager
   - Never commit credentials to version control

2. Network Security:
   - Configure firewall rules appropriately
   - Use TLS for all connections
   - Implement rate limiting for API endpoints

3. Database Security:
   - Use least-privilege database users
   - Enable SSL for database connections
   - Regularly rotate credentials

4. Monitoring:
   - Set up logging and metrics collection
   - Monitor database connection pool health
   - Track API endpoint usage and errors

## Troubleshooting

### Database Connection Issues
1. Verify environment variables are set correctly
2. Check network connectivity and firewall rules
3. Use `db_check` to verify database connection
4. Check database logs for any errors

### Application Issues
1. Check application logs (`RUST_LOG=debug`)
2. Verify all required services are running
3. Check resource usage (CPU/memory)
4. Verify configuration files are correct

## Development Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/lotaai.git
cd lotaai
```

2. Create a `.env` file:
```bash
cp .env.example .env
```

3. Start the development environment:
```bash
docker-compose up -d
```

4. Run database migrations:
```bash
docker-compose exec app cargo run --bin db_check
```

## Production Deployment

1. Build the Docker image:
```bash
docker build -t lotaai:latest .
```

2. Configure AlloyDB proxy:
   - Set up service account with necessary permissions
   - Configure proxy authentication
   - Set environment variables

3. Deploy the application:
   - Use Kubernetes for container orchestration
   - Configure health checks and monitoring
   - Set up automated backups

4. Monitor the deployment:
   - Check application logs
   - Monitor database metrics
   - Set up alerts for critical issues

## Google Cloud Secret Manager Integration

### Setting Up Secret Manager

1. Enable the Secret Manager API:
```bash
gcloud services enable secretmanager.googleapis.com
```

2. Create secrets for your application:
```bash
# Create secrets
gcloud secrets create lotaai-database-url --replication-policy="automatic"
gcloud secrets create lotaai-jwt-secret --replication-policy="automatic"

# Add secret versions
echo -n "postgresql://user:pass@host:5432/db" | \
  gcloud secrets versions add lotaai-database-url --data-file=-
echo -n "your-jwt-secret" | \
  gcloud secrets versions add lotaai-jwt-secret --data-file=-
```

3. Grant access to your service account:
```bash
# Get your service account email
export SA_EMAIL=$(gcloud iam service-accounts list \
  --filter="name:alloydb-proxy" \
  --format="value(email)")

# Grant Secret Manager access
gcloud secrets add-iam-policy-binding lotaai-database-url \
  --member="serviceAccount:$SA_EMAIL" \
  --role="roles/secretmanager.secretAccessor"
gcloud secrets add-iam-policy-binding lotaai-jwt-secret \
  --member="serviceAccount:$SA_EMAIL" \
  --role="roles/secretmanager.secretAccessor"
```

### Integrating with Docker Compose

Update your `docker-compose.yaml` to use secrets:

```yaml
services:
  app:
    environment:
      - GOOGLE_CLOUD_PROJECT=your-project-id
      - DATABASE_URL_SECRET=projects/your-project/secrets/lotaai-database-url/versions/latest
      - JWT_SECRET_SECRET=projects/your-project/secrets/lotaai-jwt-secret/versions/latest
    volumes:
      - ./config/alloydb:/secrets/alloydb:ro
```

### Production Configuration

1. Create a Kubernetes secret for GCP credentials:
```bash
kubectl create secret generic gcp-credentials \
  --from-file=credentials.json=/path/to/service-account-key.json
```

2. Update your Kubernetes deployment:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lotaai
spec:
  template:
    spec:
      containers:
      - name: lotaai
        env:
        - name: GOOGLE_CLOUD_PROJECT
          valueFrom:
            configMapKeyRef:
              name: lotaai-config
              key: project-id
        - name: GOOGLE_APPLICATION_CREDENTIALS
          value: /secrets/gcp/credentials.json
        volumeMounts:
        - name: gcp-credentials
          mountPath: /secrets/gcp
          readOnly: true
      volumes:
      - name: gcp-credentials
        secret:
          secretName: gcp-credentials
```

## Monitoring Setup

### Application Monitoring

1. Enable Google Cloud Monitoring:
```bash
gcloud services enable monitoring.googleapis.com
```

2. Configure metrics export in your application:

```toml
# Add to Cargo.toml
[dependencies]
opentelemetry = { version = "0.20", features = ["metrics"] }
opentelemetry-gcp = "0.15"
```

3. Set up custom metrics:
- Database connection pool stats
- API endpoint latency
- Request rates
- Error rates

### AlloyDB Monitoring

1. Enable AlloyDB monitoring:
```bash
gcloud services enable alloydb.googleapis.com
```

2. Configure alerts:
```bash
# CPU utilization alert
gcloud alpha monitoring policies create \
  --display-name="AlloyDB High CPU" \
  --conditions="metric.type=\"alloydb.googleapis.com/instance/cpu/utilization\" resource.type=\"alloydb_instance\" threshold=0.8"

# Connection count alert
gcloud alpha monitoring policies create \
  --display-name="AlloyDB High Connections" \
  --conditions="metric.type=\"alloydb.googleapis.com/instance/connection_count\" resource.type=\"alloydb_instance\" threshold=100"
```

### Monitoring Dashboard

Create a monitoring dashboard:
```bash
gcloud monitoring dashboards create \
  --config-from-file=monitoring/dashboard.json
```

Key metrics to monitor:
- Database connections and pool status
- Query latency and throughput
- Error rates and types
- Resource utilization (CPU, memory, disk)
- API endpoint performance

## CI/CD Integration

### GitHub Actions Setup

1. Create `.github/workflows/ci.yml`:

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run tests
      run: cargo test

    - name: Check formatting
      run: cargo fmt -- --check

    - name: Run clippy
      run: cargo clippy -- -D warnings

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3

    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v1

    - name: Login to Container Registry
      uses: docker/login-action@v1
      with:
        registry: gcr.io
        username: _json_key
        password: ${{ secrets.GCP_SA_KEY }}

    - name: Build and push
      uses: docker/build-push-action@v2
      with:
        push: true
        tags: gcr.io/${{ secrets.GCP_PROJECT }}/lotaai:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
    - uses: actions/checkout@v3

    - name: Set up GCloud
      uses: google-github-actions/setup-gcloud@v0
      with:
        service_account_key: ${{ secrets.GCP_SA_KEY }}
        project_id: ${{ secrets.GCP_PROJECT }}

    - name: Deploy to GKE
      run: |
        gcloud container clusters get-credentials your-cluster --zone your-zone
        kubectl set image deployment/lotaai \
          lotaai=gcr.io/${{ secrets.GCP_PROJECT }}/lotaai:${{ github.sha }}
```

2. Configure GitHub Secrets:
- `GCP_SA_KEY`: Service account key with necessary permissions
- `GCP_PROJECT`: Google Cloud project ID

### Deployment Environments

Configure different environments in GitHub:
1. Development
2. Staging
3. Production

Each environment should have its own:
- Kubernetes namespace
- Environment variables
- Resource limits
- Monitoring configuration

### Automated Testing

1. Unit tests run on every PR
2. Integration tests run before deployment
3. End-to-end tests run after deployment

### Security Considerations

1. Secrets Management:
   - Use GitHub Secrets for sensitive data
   - Rotate credentials regularly
   - Use separate service accounts per environment

2. Image Scanning:
   - Scan Docker images for vulnerabilities
   - Enforce security policies
   - Block deployments with critical vulnerabilities
