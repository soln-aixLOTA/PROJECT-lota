# Inference Service - Secrets Management

## Overview

The Inference Service uses HashiCorp Vault for secure secrets management, with a focus on managing NVIDIA API keys and other sensitive credentials. This document explains how secrets are managed and accessed in the service.

## Configuration

### Environment Variables

- `VAULT_ADDR`: Vault server address (default: `http://vault.lotabots.svc:8200`)
- `VAULT_ROLE`: Kubernetes authentication role (default: `inference-service`)
- `VAULT_NAMESPACE`: Vault namespace for secrets (default: `lotabots`)

### Secret Paths

Secrets are organized in Vault using the following path structure:

- `lotabots/inference-service/*`: Service-specific secrets
- `lotabots/shared/*`: Shared secrets across services
- `lotabots/nvidia/*`: NVIDIA API keys and credentials

## Usage

The Inference Service uses the `SecretsManager` class to handle all secret operations. Here's how to use it:

```python
from config import SecretsManager
import asyncio

async def example_usage():
    # Initialize the secrets manager
    secrets_manager = SecretsManager()
    await secrets_manager.init()
    
    try:
        # Get NVIDIA API key
        nvidia_key = await secrets_manager.get_secret("nvidia/api-key")
        
        # Get a dynamic secret
        model_credentials = await secrets_manager.get_dynamic_secret(
            "inference-service/model-credentials"
        )
        
    finally:
        # Clean up resources
        await secrets_manager.cleanup()

# Run the example
asyncio.run(example_usage())
```

## Features

- **Kubernetes Authentication**: Uses Kubernetes service account tokens for authentication
- **Secret Caching**: Implements efficient caching with TTL for static secrets
- **Lease Management**: Automatically manages and renews secret leases
- **Dynamic Secrets**: Supports both static and dynamic secrets
- **Error Handling**: Comprehensive error handling and logging
- **Async Support**: Fully asynchronous implementation

## Security Considerations

1. **Least Privilege**: The service uses a dedicated Vault policy that restricts access to only required secrets
2. **Secure Authentication**: Uses Kubernetes service account tokens for authentication
3. **Secret Rotation**: Supports automatic rotation of dynamic secrets
4. **Audit Logging**: All secret access is logged for audit purposes

## Vault Policy

The Inference Service uses the following Vault policy:

```hcl
# Inference service policy
path "lotabots/inference-service/*" {
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

# Allow access to NVIDIA API keys
path "lotabots/nvidia/*" {
  capabilities = ["read"]
}
```

## Dependencies

Required Python packages:

```
hvac>=1.0.0
cachetools>=5.0.0
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

4. **Cache Issues**:
   - Check cache TTL settings
   - Monitor cache hit/miss rates
   - Verify memory usage

## Monitoring

The service exports the following metrics for monitoring:

- `vault_secret_requests_total`: Total number of secret requests
- `vault_secret_cache_hits_total`: Number of cache hits
- `vault_lease_renewal_errors_total`: Number of lease renewal errors
- `vault_auth_failures_total`: Number of authentication failures

These metrics can be collected by Prometheus and visualized in Grafana.

## Best Practices

1. **Error Handling**:
   ```python
   try:
       secret = await secrets_manager.get_secret("path/to/secret")
   except Exception as e:
       logger.error(f"Failed to get secret: {e}")
       # Implement appropriate fallback or error handling
   ```

2. **Resource Cleanup**:
   ```python
   secrets_manager = SecretsManager()
   try:
       await secrets_manager.init()
       # Use secrets manager
   finally:
       await secrets_manager.cleanup()
   ```

3. **Lease Management**:
   - Monitor lease expiration
   - Implement proper error handling for lease renewal failures
   - Use appropriate TTL values for different types of secrets 