# API Gateway - Secrets Management

## Overview

The API Gateway uses HashiCorp Vault for secure secrets management. This document explains how secrets are managed, accessed, and configured in the service.

## Configuration

### Environment Variables

- `VAULT_ADDR`: Vault server address (default: `http://vault.lotabots.svc:8200`)
- `VAULT_ROLE`: Kubernetes authentication role (default: `api-gateway`)
- `VAULT_NAMESPACE`: Vault namespace for secrets (default: `lotabots`)

### Secret Paths

Secrets are organized in Vault using the following path structure:

- `lotabots/api-gateway/*`: API Gateway specific secrets
- `lotabots/shared/*`: Shared secrets across services
- `lotabots/database/*`: Database credentials

## Usage

The API Gateway uses the `SecretsManager` to handle all secret operations. Here's how to use it:

```rust
use crate::secrets::SecretsManager;

async fn example_usage() -> Result<(), Error> {
    let config = SecretConfig {
        vault_addr: std::env::var("VAULT_ADDR")?,
        vault_role: std::env::var("VAULT_ROLE")?,
        namespace: std::env::var("VAULT_NAMESPACE")?,
    };
    
    let secrets_manager = SecretsManager::new(config).await?;
    
    // Get a static secret
    let api_key = secrets_manager.get_secret("api-gateway/api-key").await?;
    
    // Get a dynamic secret (e.g., database credentials)
    let db_creds = secrets_manager.get_dynamic_secret("database/creds/api-gateway").await?;
    
    Ok(())
}
```

## Features

- **Kubernetes Authentication**: Uses Kubernetes service account tokens for authentication
- **Secret Caching**: Implements efficient caching with TTL
- **Lease Management**: Automatically manages and renews secret leases
- **Dynamic Secrets**: Supports both static and dynamic secrets
- **Error Handling**: Comprehensive error handling and logging

## Security Considerations

1. **Least Privilege**: The service uses a dedicated Vault policy that restricts access to only required secrets
2. **Secure Authentication**: Uses Kubernetes service account tokens for authentication
3. **Secret Rotation**: Supports automatic rotation of dynamic secrets
4. **Audit Logging**: All secret access is logged for audit purposes

## Vault Policy

The API Gateway uses the following Vault policy:

```hcl
# API Gateway service policy
path "lotabots/api-gateway/*" {
  capabilities = ["read"]
}

path "lotabots/shared/*" {
  capabilities = ["read"]
}

path "auth/token/renew-self" {
  capabilities = ["update"]
}

path "sys/leases/renew" {
  capabilities = ["update"]
}

path "sys/renew/*" {
  capabilities = ["update"]
}
```

## Troubleshooting

Common issues and solutions:

1. **Authentication Failures**:
   - Verify the Kubernetes service account token is mounted
   - Check the Vault role binding is correct
   - Ensure network policies allow communication with Vault

2. **Secret Access Denied**:
   - Verify the Vault policy is correctly applied
   - Check the secret path matches the policy
   - Ensure the service account has the correct role binding

3. **Lease Renewal Failures**:
   - Check Vault server connectivity
   - Verify the lease TTL is within allowed limits
   - Ensure the policy allows lease renewal

## Monitoring

The service exports the following metrics for monitoring:

- `vault_secret_requests_total`: Total number of secret requests
- `vault_secret_cache_hits_total`: Number of cache hits
- `vault_lease_renewal_errors_total`: Number of lease renewal errors

These metrics can be collected by Prometheus and visualized in Grafana. 