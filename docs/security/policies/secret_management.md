# Secret Management Policy

## Table of Contents

1. [Overview](#overview)
2. [SecretManager Trait](#secretmanager-trait)
   - [Implementations](#implementations)
     - AWS Secrets Manager
     - HashiCorp Vault
     - Azure Key Vault
     - Google Cloud Secret Manager
3. [Secret Rotation](#secret-rotation)
   - [Automated Secret Rotation](#automated-secret-rotation)
4. [Monitoring and Alerting](#monitoring-and-alerting)
   - [Metrics](#metrics)
   - [Operation Timing](#operation-timing)
   - [Alert Configuration](#alert-configuration)
5. [Testing](#testing)
   - [Unit Tests](#unit-tests)
   - [Integration Tests](#integration-tests)
6. [Security Best Practices](#security-best-practices)
7. [Error Handling](#error-handling)
   - [Common Error Types](#common-error-types)
   - [Handling Specific Errors](#handling-specific-errors)
8. [Configuration Details](#configuration-details)
   - [AWS Secrets Manager Configuration](#aws-secrets-manager-configuration)
   - [HashiCorp Vault Configuration](#hashicorp-vault-configuration)
   - [Environment-Specific Configuration](#environment-specific-configuration)
9. [Audit Logging](#audit-logging)
   - [Event Types](#event-types)
   - [Audit Log Storage](#audit-log-storage)
10. [Compliance Mapping](#compliance-mapping)
    - [NIST 800-53 Controls](#nist-800-53-controls)
    - [SOC 2 Type 2](#soc-2-type-2)
    - [ISO 27001](#iso-27001)
    - [HIPAA](#hipaa)
11. [Enhanced Alerting Examples](#enhanced-alerting-examples)
    - [Google Cloud Monitoring](#google-cloud-monitoring)
    - [Datadog](#datadog)

This document outlines LOTA AI's policies and procedures for managing secrets, credentials, and sensitive configuration values using our `SecretManager` trait and its implementations.

## SecretManager Trait

The `SecretManager` trait provides a standardized interface for interacting with different secrets managers:

```rust
#[async_trait]
pub trait SecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn create_secret(&self, secret_name: &str, secret_value: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn revoke_secret(&self, secret_name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

### Implementations

#### AWS Secrets Manager

```rust
use aws_sdk_secretsmanager::{Client, Error};

pub struct AwsSecretManager;

#[async_trait]
impl SecretManager for AwsSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Implementation details...
    }
}
```

#### HashiCorp Vault

```rust
pub struct VaultSecretManager;

#[async_trait]
impl SecretManager for VaultSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Format: "path/to/secret#key"
        let parts: Vec<&str> = secret_name.splitn(2, '#').collect();
        if parts.len() != 2 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid secret name format for Vault. Expected 'path/to/secret#key'",
            )));
        }
        // Implementation details...
    }
}
```

#### Azure Key Vault

```rust
pub struct AzureSecretManager;

#[async_trait]
impl SecretManager for AzureSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Implementation details...
    }
}
```

#### Google Cloud Secret Manager

```rust
pub struct GoogleSecretManager {
    pub project_id: String,
}

#[async_trait]
impl SecretManager for GoogleSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Format: "secret_id/version_id"
        let parts: Vec<&str> = secret_name.splitn(2, '/').collect();
        // Implementation details...
    }
}
```

## Secret Rotation

The `SecretRotation` struct provides functionality for rotating secrets:

```rust
pub struct SecretRotation<T: SecretManager> {
    secret_manager: T,
}

impl<T: SecretManager> SecretRotation<T> {
    pub fn new(secret_manager: T) -> Self {
        Self { secret_manager }
    }

    pub async fn rotate_secret(&self, secret_name: &str) -> Result<(), Box<dyn Error>> {
        // Generate new secret
        let new_secret = self.generate_secure_secret()?;

        // Store new secret
        self.secret_manager.create_secret(secret_name, &new_secret).await?;

        // Update application configuration
        self.update_application_config(secret_name, &new_secret).await?;

        // Revoke old secret
        self.secret_manager.revoke_secret(secret_name).await?;

        Ok(())
    }
}
```

### Automated Secret Rotation

To automate secret rotation, you can:

1. Use a cron job:
```bash
0 0 * * * /usr/local/bin/rotate-secrets
```

2. Use serverless functions (e.g., AWS Lambda):
```rust
#[lambda_runtime::handler_fn]
async fn handler(event: LambdaEvent<()>) -> Result<(), Error> {
    let secret_manager = AwsSecretManager::new();
    let rotation = SecretRotation::new(secret_manager);
    rotation.rotate_secret("my-secret").await?;
    Ok(())
}
```

## Monitoring and Alerting

### Metrics

The `SecretMetrics` struct provides monitoring capabilities:

```rust
pub struct SecretMetrics;

impl SecretMetrics {
    pub fn record_secret_retrieval(secret_name: &str, success: bool) {
        if success {
            counter!("secret_retrieval_success_count", 1, "secret_name" => secret_name.to_string());
        } else {
            counter!("secret_retrieval_failure_count", 1, "secret_name" => secret_name.to_string());
        }
    }

    pub fn record_secret_rotation(secret_name: &str, success: bool) {
        if success {
            counter!("secret_rotation_success_count", 1, "secret_name" => secret_name.to_string());
        } else {
            counter!("secret_rotation_failure_count", 1, "secret_name" => secret_name.to_string());
        }
    }
}
```

### Operation Timing

Use the `SecretOperationTimer` for tracking operation durations:

```rust
let _timer = SecretOperationTimer::new("get_secret");
// Operation will be timed automatically
```

### Alert Configuration

Example Prometheus alert rules:

```yaml
groups:
- name: secret-management
  rules:
  - alert: HighSecretRetrievalFailureRate
    expr: rate(secret_retrieval_failure_count[5m]) > 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High rate of secret retrieval failures

  - alert: SecretRotationFailure
    expr: secret_rotation_failure_count > 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: Secret rotation failure detected
```

## Testing

### Unit Tests

Use `mockall` for testing secret manager implementations:

```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        pub TestSecretManager {}

        #[async_trait]
        impl SecretManager for TestSecretManager {
            async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn Error>>;
        }
    }

    #[tokio::test]
    async fn test_get_secret_success() {
        let mut mock = MockTestSecretManager::new();
        mock.expect_get_secret()
            .with(eq("test_secret"))
            .times(1)
            .returning(|_| Ok("test_value".to_string()));

        let secret = mock.get_secret("test_secret").await.unwrap();
        assert_eq!(secret, "test_value");
    }
}
```

### Integration Tests

For integration testing with actual secrets managers:

1. Set up test environments for each secrets manager
2. Create test secrets
3. Run tests against these environments
4. Clean up test secrets after testing

## Security Best Practices

1. **Never commit secrets to version control**
   - Use environment variables for local development
   - Use secrets managers for production
   - Add sensitive files to `.gitignore`

2. **Implement proper access control**
   - Use role-based access control (RBAC)
   - Implement the principle of least privilege
   - Regularly audit access logs

3. **Regular secret rotation**
   - Rotate secrets on a schedule
   - Rotate immediately if compromise is suspected
   - Use automated rotation where possible

4. **Comprehensive monitoring**
   - Monitor all secret access attempts
   - Set up alerts for suspicious activity
   - Regularly review access patterns

5. **Pre-commit hooks**
   - Use tools like `detect-secrets` or `gitleaks`
   - Block commits containing potential secrets
   - Regularly update detection patterns

## Error Handling

### Common Error Types

```rust
#[derive(Debug)]
pub enum SecretError {
    NotFound(String),
    AuthenticationFailed(String),
    PermissionDenied(String),
    NetworkError(String),
    InvalidFormat(String),
    Other(String),
}

impl std::error::Error for SecretError {}
```

### Handling Specific Errors

#### Network Errors
```rust
async fn handle_network_error(err: SecretError) -> Result<(), Box<dyn Error>> {
    match err {
        SecretError::NetworkError(msg) => {
            tracing::error!("Network error accessing secret: {}", msg);
            metrics::counter!("secret_network_errors", 1);
            // Implement retry logic
            Ok(())
        }
        _ => Err(Box::new(err)),
    }
}
```

#### Authentication Errors
```rust
async fn handle_auth_error(err: SecretError) -> Result<(), Box<dyn Error>> {
    match err {
        SecretError::AuthenticationFailed(msg) => {
            tracing::error!("Authentication failed: {}", msg);
            metrics::counter!("secret_auth_failures", 1);
            // Trigger credential refresh
            Ok(())
        }
        _ => Err(Box::new(err)),
    }
}
```

## Configuration Details

### AWS Secrets Manager Configuration
```rust
pub struct AwsSecretsConfig {
    pub region: String,
    pub credentials_profile: Option<String>,
    pub endpoint_url: Option<String>,
}

impl AwsSecretManager {
    pub fn new(config: AwsSecretsConfig) -> Self {
        // Implementation details...
    }
}
```

### HashiCorp Vault Configuration
```rust
pub struct VaultConfig {
    pub address: String,
    pub token: String,
    pub namespace: Option<String>,
}

impl VaultSecretManager {
    pub fn new(config: VaultConfig) -> Self {
        // Implementation details...
    }
}
```

### Environment-Specific Configuration
```toml
# config/production.toml
[secrets]
provider = "aws"
region = "us-east-1"
credentials_profile = "production"

# config/development.toml
[secrets]
provider = "vault"
address = "http://localhost:8200"
token = "${VAULT_TOKEN}"
```

## Audit Logging

### Event Types
```rust
#[derive(Debug)]
pub enum SecretEvent {
    Retrieval { secret_name: String, success: bool },
    Creation { secret_name: String },
    Rotation { secret_name: String, success: bool },
    Deletion { secret_name: String },
}

impl SecretEvent {
    pub fn log(&self, user: &str) {
        let timestamp = chrono::Utc::now();
        let event_type = match self {
            SecretEvent::Retrieval { .. } => "retrieval",
            SecretEvent::Creation { .. } => "creation",
            SecretEvent::Rotation { .. } => "rotation",
            SecretEvent::Deletion { .. } => "deletion",
        };

        tracing::info!(
            timestamp = timestamp.to_rfc3339(),
            event_type = event_type,
            user = user,
            secret_name = self.secret_name(),
            success = self.success(),
            "Secret management event"
        );
    }
}
```

### Audit Log Storage
```rust
pub struct AuditLogger {
    storage: Box<dyn AuditStorage>,
}

impl AuditLogger {
    pub async fn log_event(&self, event: SecretEvent, user: &str) -> Result<(), Box<dyn Error>> {
        // Store event in secure storage
        self.storage.store_event(event, user).await?;
        Ok(())
    }
}
```

## Compliance Mapping

### NIST 800-53 Controls

| Control ID | Description | Implementation |
|------------|-------------|----------------|
| AC-3 | Access Enforcement | Role-based access control in `SecretManager` implementations |
| AU-2 | Audit Events | Comprehensive audit logging via `AuditLogger` |
| SC-12 | Cryptographic Key Management | Secret rotation via `SecretRotation` struct |
| SC-28 | Protection of Information at Rest | Secrets stored in approved managers (AWS, Vault, etc.) |

### SOC 2 Type 2

| Trust Service Criteria | Implementation |
|-----------------------|----------------|
| CC6.1 | Role-based access control |
| CC6.2 | Audit logging of all secret operations |
| CC7.1 | Secret rotation and monitoring |
| CC7.2 | Alert configuration for suspicious activities |

### ISO 27001

| Control | Implementation |
|---------|----------------|
| A.9.2 | User access management via RBAC |
| A.10.1 | Cryptographic controls via secret managers |
| A.12.4 | Logging and monitoring |
| A.18.1.3 | Protection of records |

### HIPAA

| Requirement | Implementation |
|-------------|----------------|
| Access Control | Role-based access in `SecretManager` |
| Audit Controls | Comprehensive audit logging |
| Integrity | Secret rotation and validation |
| Transmission Security | Secure secret transmission |

## Enhanced Alerting Examples

### Google Cloud Monitoring

```yaml
# alerts.yaml
alertPolicies:
- displayName: "Secret Retrieval Failures"
  conditions:
  - displayName: "High failure rate"
    conditionThreshold:
      filter: metric.type="custom.googleapis.com/secret_retrieval_failure_count"
      aggregations:
      - alignmentPeriod: 300s
        perSeriesAligner: ALIGN_RATE
      comparison: COMPARISON_GT
      thresholdValue: 0.1
      duration: 300s
  alertStrategy:
    autoClose: 7200s
  notificationChannels:
  - "projects/my-project/notificationChannels/my-channel"
```

### Datadog

```yaml
# datadog-monitor.yaml
monitors:
- name: "Secret Rotation Failure"
  type: "metric alert"
  query: "sum(last_5m):sum:secret_rotation_failure_count{*} > 0"
  message: "Secret rotation failure detected! @security-team"
  tags:
    - "service:secrets"
    - "severity:critical"
```
