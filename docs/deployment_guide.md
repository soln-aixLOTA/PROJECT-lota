# LotaBots Deployment Guide

## Overview

This guide provides detailed instructions for deploying the LotaBots platform across different environments. It covers infrastructure setup, configuration management, and deployment strategies.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Infrastructure Provisioning](#infrastructure-provisioning)
- [Application Deployment](#application-deployment)
- [Configuration Management](#configuration-management)
- [Deployment Verification](#deployment-verification)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools
```bash
# Versions
terraform >= 1.0.0
kubectl >= 1.24.0
helm >= 3.8.0
aws-cli >= 2.7.0
docker >= 20.10.0
```

### Access Requirements
- AWS IAM credentials with appropriate permissions
- Kubernetes cluster access
- Docker registry access

## Environment Setup

### Development Environment
```bash
# Clone repository
git clone https://github.com/lotabots/platform.git
cd platform

# Install dependencies
cargo install --path .

# Set up local environment
cp .env.example .env
docker-compose up -d
```

### Production Environment Variables
```bash
# Core Settings
ENVIRONMENT=production
LOG_LEVEL=info
RUST_LOG=info

# Database
DATABASE_URL=postgresql://user:pass@rds-instance:5432/lotabots
DB_POOL_SIZE=10

# Redis
REDIS_URL=redis://elasticache-instance:6379
REDIS_POOL_SIZE=20

# AWS
AWS_REGION=us-west-2
S3_BUCKET=lotabots-production
```

## Infrastructure Provisioning

### AWS Infrastructure (Terraform)
```hcl
# VPC Configuration
module "vpc" {
  source = "terraform-aws-modules/vpc/aws"

  name = "lotabots-vpc"
  cidr = "10.0.0.0/16"

  azs             = ["us-west-2a", "us-west-2b", "us-west-2c"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  enable_nat_gateway = true
  single_nat_gateway = false
}

# EKS Cluster
module "eks" {
  source = "terraform-aws-modules/eks/aws"

  cluster_name    = "lotabots-cluster"
  cluster_version = "1.24"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    general = {
      desired_size = 2
      min_size     = 1
      max_size     = 4

      instance_types = ["t3.xlarge"]
      capacity_type  = "ON_DEMAND"
    }
  }
}
```

### Kubernetes Resources

#### Base Configuration
```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: lotabots
  labels:
    name: lotabots
    environment: production
```

#### Service Deployment
```yaml
# api-gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: lotabots
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: api-gateway
        image: lotabots/api-gateway:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: "1"
            memory: "2Gi"
          limits:
            cpu: "2"
            memory: "4Gi"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 20
```

## Application Deployment

### Deployment Process Flow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Git as Git Repository
    participant CI as CI/CD Pipeline
    participant Reg as Docker Registry
    participant K8s as Kubernetes

    Dev->>Git: Push Code
    Git->>CI: Trigger Pipeline
    CI->>CI: Build & Test
    CI->>CI: Security Scan
    CI->>Reg: Push Image
    CI->>K8s: Deploy to Staging
    Note over K8s: Run Integration Tests
    CI->>K8s: Deploy to Production
```

### Deployment Commands
```bash
# Deploy to staging
kubectl apply -f k8s/staging/

# Verify staging deployment
kubectl get pods -n lotabots-staging
kubectl logs -f deployment/api-gateway -n lotabots-staging

# Deploy to production
kubectl apply -f k8s/production/

# Verify production deployment
kubectl get pods -n lotabots
kubectl logs -f deployment/api-gateway -n lotabots
```

## Configuration Management

### Helm Values
```yaml
# values.yaml
global:
  environment: production
  domain: api.lotabots.com

api-gateway:
  replicas: 3
  resources:
    requests:
      cpu: "1"
      memory: "2Gi"
    limits:
      cpu: "2"
      memory: "4Gi"

  config:
    logLevel: info
    metricsEnabled: true

  ingress:
    enabled: true
    annotations:
      kubernetes.io/ingress-class: nginx
      cert-manager.io/cluster-issuer: letsencrypt-prod

database:
  host: rds-instance
  port: 5432
  poolSize: 10
```

### Secret Management
```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: lotabots-secrets
  namespace: lotabots
type: Opaque
data:
  DATABASE_URL: <base64-encoded-url>
  REDIS_URL: <base64-encoded-url>
  JWT_SECRET: <base64-encoded-secret>
```

## Deployment Verification

### Health Check Endpoints
```bash
# API Gateway
curl https://api.lotabots.com/health

# Authentication Service
curl https://api.lotabots.com/auth/health

# Hardware Attestation
curl https://api.lotabots.com/attestation/health
```

### Monitoring Checks
```bash
# Check Prometheus metrics
curl https://api.lotabots.com/metrics

# View service logs
kubectl logs -f -l app=api-gateway -n lotabots

# Check pod status
kubectl get pods -n lotabots
kubectl describe pod <pod-name> -n lotabots
```

## Troubleshooting

### Common Issues

#### Database Connection Issues
```bash
# Check database connectivity
kubectl exec -it <pod-name> -n lotabots -- pg_isready -h $DB_HOST

# View database logs
kubectl logs -f deployment/api-gateway -n lotabots | grep "database"
```

#### Pod Startup Issues
```bash
# Check pod events
kubectl describe pod <pod-name> -n lotabots

# Check container logs
kubectl logs -f <pod-name> -n lotabots
```

#### Resource Issues
```bash
# Check resource usage
kubectl top pods -n lotabots
kubectl top nodes

# Check pod resource limits
kubectl describe pod <pod-name> -n lotabots | grep -A 5 "Limits:"
```

### Recovery Procedures

#### Service Recovery
```bash
# Restart deployment
kubectl rollout restart deployment <deployment-name> -n lotabots

# Rollback deployment
kubectl rollout undo deployment <deployment-name> -n lotabots

# Scale deployment
kubectl scale deployment <deployment-name> -n lotabots --replicas=3
```

#### Database Recovery
```bash
# Connect to database
kubectl exec -it <pod-name> -n lotabots -- psql $DATABASE_URL

# Check database status
\dt
\d+ table_name
```

### Useful Commands
```bash
# View all resources in namespace
kubectl get all -n lotabots

# View pod logs with timestamps
kubectl logs -f <pod-name> -n lotabots --timestamps

# View pod metrics
kubectl top pod <pod-name> -n lotabots

# Execute commands in container
kubectl exec -it <pod-name> -n lotabots -- /bin/sh
```
