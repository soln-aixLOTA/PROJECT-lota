# LotaBots Secrets Management

## Overview

This document provides a comprehensive guide to secrets management in the LotaBots platform. All services use HashiCorp Vault for secure secrets management, with a standardized approach to configuration, authentication, and access control.

## Architecture

### Components

1. **HashiCorp Vault**
   - High-availability deployment (3 replicas)
   - Raft storage backend
   - Kubernetes authentication
   - Audit logging enabled

2. **Common Libraries**
   - `lotabots-config`: Configuration management
   - `lotabots-secrets`: Secret management interfaces and utilities

3. **Service Integration**
   - Kubernetes service account authentication
   - Policy-based access control
   - Automatic lease management
   - Metrics collection

## Configuration

### Environment Variables

All services use these standard environment variables:

```bash
VAULT_ADDR="http://vault.lotabots.svc:8200"
VAULT_ROLE="<service-name>"
VAULT_NAMESPACE="lotabots"
```

### Secret Paths

Secrets are organized using the following path structure:

```
lotabots/
├── shared/           # Shared secrets
├── api-gateway/      # API Gateway secrets
├── inference-service/# Inference Service secrets
├── user-auth/       # User Authentication secrets
├── nvidia/          # NVIDIA credentials
├── database/        # Database credentials
├── models/          # Model credentials
├── metrics/         # Metrics data
└── sessions/        # Session data
```

## Service Integration

### Rust Services

```rust
use lotabots_config::SecretsConfig;
use lotabots_secrets::{SecretManager, MetricsCollector};

async fn setup_secrets() -> Result<(), Error> {
    let config = SecretsConfig::from_env()?;
    let metrics = PrometheusMetrics::new("vault");
    
    let secrets = SecretManagerBuilder::new()
        .with_config(config)
        .with_metrics(metrics)
        .build()
        .await?;
        
    secrets.init().await?;
    Ok(())
}
```

### Python Services

```python
from lotabots.config import SecretsConfig
from lotabots.secrets import SecretManager

async def setup_secrets():
    config = SecretsConfig.from_env()
    metrics = PrometheusMetrics(prefix="vault")
    
    secrets = SecretManagerBuilder.new()\
        .with_config(config)\
        .with_metrics(metrics)\
        .build()
        
    await secrets.init()
```

## Security Model

### Authentication

1. **Kubernetes Authentication**
   - Services authenticate using their Kubernetes service account tokens
   - Each service has a dedicated Vault role
   - Role bindings are managed through Kubernetes RBAC

2. **Policy Hierarchy**
   - Base policy (`base.hcl`): Common permissions for all services
   - Service-specific policies: Additional permissions based on service needs

### Access Control

1. **Least Privilege**
   - Services can only access their designated secret paths
   - Read-only access where possible
   - Fine-grained control through policy capabilities

2. **Dynamic Secrets**
   - Database credentials are dynamically generated
   - Automatic credential rotation
   - Lease management and renewal

## Monitoring and Auditing

### Metrics

Standard metrics exported by all services:

- `vault_secret_requests_total`
- `vault_secret_cache_hits_total`
- `vault_lease_renewal_errors_total`
- `vault_auth_failures_total`

### Audit Logging

Vault audit logs capture:
- All secret access attempts
- Authentication events
- Policy changes
- Lease operations

## Best Practices

1. **Secret Access**
   ```rust
   // Good: Use the secret manager
   let api_key = secrets.get_secret("api-gateway/api-key").await?;
   
   // Bad: Direct environment variable access
   let api_key = std::env::var("API_KEY")?;
   ```

2. **Error Handling**
   ```rust
   match secrets.get_secret("path/to/secret").await {
       Ok(secret) => // Use secret,
       Err(SecretError::NotFound(_)) => // Handle missing secret,
       Err(SecretError::AccessDenied(_)) => // Handle access denied,
       Err(e) => // Handle other errors,
   }
   ```

3. **Resource Cleanup**
   ```rust
   let secrets = SecretManager::new(config).await?;
   
   // Ensure cleanup runs
   tokio::spawn(async move {
       if let Err(e) = secrets.cleanup().await {
           error!("Failed to clean up secrets: {}", e);
       }
   });
   ```

## Troubleshooting

Common issues and solutions:

1. **Authentication Failures**
   - Verify service account token mounting
   - Check role binding configuration
   - Validate network policies

2. **Secret Access Denied**
   - Review policy configuration
   - Verify secret path
   - Check service account permissions

3. **Lease Renewal Failures**
   - Monitor lease TTLs
   - Check network connectivity
   - Verify renewal permissions

## Migration Guide

1. **Scan for Existing Secrets**
   ```bash
   python scripts/secret_inventory.py /path/to/codebase
   ```

2. **Review and Adjust Mapping**
   ```bash
   # Edit secret_mapping.yaml as needed
   ```

3. **Migrate Secrets**
   ```bash
   python scripts/migrate_to_vault.py secret_mapping.yaml
   ```

4. **Verify Migration**
   ```bash
   # Check migration status
   python scripts/migrate_to_vault.py --verify secret_mapping.yaml
   ```

## Support and Maintenance

1. **Regular Tasks**
   - Monitor audit logs
   - Review access patterns
   - Update policies as needed
   - Rotate root credentials

2. **Emergency Procedures**
   - Seal Vault in case of breach
   - Revoke compromised tokens
   - Rotate affected secrets

## References

- [HashiCorp Vault Documentation](https://www.vaultproject.io/docs)
- [Kubernetes Authentication](https://www.vaultproject.io/docs/auth/kubernetes)
- [Policy Documentation](https://www.vaultproject.io/docs/concepts/policies)
- [API Documentation](https://www.vaultproject.io/api-docs) 