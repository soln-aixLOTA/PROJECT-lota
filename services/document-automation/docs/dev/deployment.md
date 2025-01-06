# Deployment Guide

This guide covers different deployment options for the Document Automation Service.

## Docker Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- Access to container registry (optional)

### Configuration

Create a `.env` file:

```env
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info

# Database
DATABASE_URL=postgresql://user:password@db:5432/docautomation
POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=docautomation

# Storage
STORAGE_PROVIDER=s3
S3_BUCKET=documents
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key

# Security
JWT_SECRET=your_jwt_secret
TOKEN_EXPIRATION_HOURS=24
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: "3.8"

services:
  app:
    build: .
    ports:
      - "8080:8080"
    env_file: .env
    depends_on:
      - db
    volumes:
      - documents:/data/documents
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  db:
    image: postgres:14-alpine
    env_file: .env
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  documents:
  postgres_data:
```

### Building and Running

```bash
# Build the image
docker compose build

# Start services
docker compose up -d

# Check logs
docker compose logs -f

# Run migrations
docker compose exec app sqlx migrate run
```

## Kubernetes Deployment

### Prerequisites

- Kubernetes 1.22+
- kubectl configured
- Helm 3+ (optional)

### Configuration

1. Create ConfigMap (`config.yaml`):

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: docautomation-config
data:
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "8080"
  RUST_LOG: "info"
  STORAGE_PROVIDER: "s3"
  S3_BUCKET: "documents"
  AWS_REGION: "us-east-1"
```

2. Create Secrets (`secrets.yaml`):

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: docautomation-secrets
type: Opaque
stringData:
  DATABASE_URL: postgresql://user:password@postgres:5432/docautomation
  JWT_SECRET: your_jwt_secret
  AWS_ACCESS_KEY_ID: your_access_key
  AWS_SECRET_ACCESS_KEY: your_secret_key
```

3. Create Deployment (`deployment.yaml`):

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: docautomation
spec:
  replicas: 3
  selector:
    matchLabels:
      app: docautomation
  template:
    metadata:
      labels:
        app: docautomation
    spec:
      containers:
        - name: docautomation
          image: docautomation:latest
          ports:
            - containerPort: 8080
          envFrom:
            - configMapRef:
                name: docautomation-config
            - secretRef:
                name: docautomation-secrets
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          resources:
            requests:
              memory: "256Mi"
              cpu: "100m"
            limits:
              memory: "512Mi"
              cpu: "500m"
```

4. Create Service (`service.yaml`):

```yaml
apiVersion: v1
kind: Service
metadata:
  name: docautomation
spec:
  selector:
    app: docautomation
  ports:
    - port: 80
      targetPort: 8080
  type: LoadBalancer
```

### Deployment Steps

```bash
# Apply configurations
kubectl apply -f config.yaml
kubectl apply -f secrets.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml

# Check status
kubectl get pods
kubectl get services

# View logs
kubectl logs -l app=docautomation

# Run migrations
kubectl exec -it $(kubectl get pod -l app=docautomation -o name | head -1) -- sqlx migrate run
```

## Bare Metal Deployment

### Prerequisites

- Linux server (Ubuntu 20.04+ recommended)
- PostgreSQL 12+
- Nginx (optional, for reverse proxy)
- Systemd

### Installation

1. Install dependencies:

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install required packages
sudo apt install -y build-essential pkg-config libssl-dev postgresql nginx
```

2. Create service user:

```bash
sudo useradd -r -s /bin/false docautomation
```

3. Create service directories:

```bash
sudo mkdir -p /opt/docautomation/{bin,config,data}
sudo chown -R docautomation:docautomation /opt/docautomation
```

4. Copy binary and configuration:

```bash
sudo cp target/release/document-automation /opt/docautomation/bin/
sudo cp config/production.toml /opt/docautomation/config/
```

5. Create systemd service (`/etc/systemd/system/docautomation.service`):

```ini
[Unit]
Description=Document Automation Service
After=network.target postgresql.service

[Service]
Type=simple
User=docautomation
Group=docautomation
ExecStart=/opt/docautomation/bin/document-automation
WorkingDirectory=/opt/docautomation
Environment=CONFIG_FILE=/opt/docautomation/config/production.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

6. Configure Nginx (optional):

```nginx
server {
    listen 80;
    server_name docautomation.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Starting the Service

```bash
# Enable and start the service
sudo systemctl enable docautomation
sudo systemctl start docautomation

# Check status
sudo systemctl status docautomation

# View logs
sudo journalctl -u docautomation -f
```

## Monitoring & Maintenance

### Health Checks

The service exposes a `/health` endpoint that returns:

- Service status
- Database connectivity
- Storage backend status

### Metrics

Prometheus metrics are available at `/metrics`:

- Request counts and latencies
- Document upload/download statistics
- Storage usage
- Error rates

### Backup Strategy

1. **Database Backups**

   ```bash
   # Automated backup script
   pg_dump docautomation > backup_$(date +%Y%m%d).sql
   ```

2. **Document Storage Backups**
   - For S3: Enable versioning and cross-region replication
   - For local storage: Regular filesystem backups

### Scaling Considerations

1. **Horizontal Scaling**

   - Deploy multiple instances behind a load balancer
   - Use sticky sessions if needed
   - Scale database with read replicas

2. **Vertical Scaling**

   - Increase CPU/memory limits
   - Optimize database configuration
   - Tune worker threads

3. **Storage Scaling**
   - Implement storage sharding
   - Use CDN for frequent downloads
   - Consider hot/cold storage tiers
