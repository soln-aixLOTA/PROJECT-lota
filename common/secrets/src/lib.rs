use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("Secret not found: {0}")]
    NotFound(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Lease expired: {0}")]
    LeaseExpired(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Secret {
    pub value: String,
    pub lease_id: Option<String>,
    pub lease_duration: Option<u64>,
    pub renewable: bool,
}

#[async_trait]
pub trait SecretManager: Send + Sync + Debug {
    /// Initialize the secret manager
    async fn init(&self) -> Result<(), SecretError>;
    
    /// Get a static secret
    async fn get_secret(&self, path: &str) -> Result<Secret, SecretError>;
    
    /// Get a dynamic secret
    async fn get_dynamic_secret(&self, path: &str) -> Result<Secret, SecretError>;
    
    /// Set a secret
    async fn set_secret(&self, path: &str, value: &str) -> Result<(), SecretError>;
    
    /// Delete a secret
    async fn delete_secret(&self, path: &str) -> Result<(), SecretError>;
    
    /// Cleanup resources
    async fn cleanup(&self) -> Result<(), SecretError>;
}

#[async_trait]
pub trait LeaseManager: Send + Sync + Debug {
    /// Register a lease for renewal
    async fn register_lease(&self, lease_id: String, ttl: u64) -> Result<(), SecretError>;
    
    /// Renew a lease
    async fn renew_lease(&self, lease_id: &str) -> Result<u64, SecretError>;
    
    /// Revoke a lease
    async fn revoke_lease(&self, lease_id: &str) -> Result<(), SecretError>;
}

#[async_trait]
pub trait MetricsCollector: Send + Sync + Debug {
    /// Record a secret request
    async fn record_secret_request(&self, path: &str, success: bool);
    
    /// Record a cache hit
    async fn record_cache_hit(&self, path: &str);
    
    /// Record a lease renewal
    async fn record_lease_renewal(&self, success: bool);
    
    /// Record authentication attempt
    async fn record_auth_attempt(&self, success: bool);
}

#[derive(Debug)]
pub struct NoopMetricsCollector;

#[async_trait]
impl MetricsCollector for NoopMetricsCollector {
    async fn record_secret_request(&self, _path: &str, _success: bool) {}
    async fn record_cache_hit(&self, _path: &str) {}
    async fn record_lease_renewal(&self, _success: bool) {}
    async fn record_auth_attempt(&self, _success: bool) {}
}

#[derive(Debug)]
pub struct SecretManagerBuilder<M: MetricsCollector> {
    config: Option<lotabots_config::SecretsConfig>,
    metrics: Option<M>,
}

impl<M: MetricsCollector> Default for SecretManagerBuilder<M> {
    fn default() -> Self {
        Self {
            config: None,
            metrics: None,
        }
    }
}

impl<M: MetricsCollector> SecretManagerBuilder<M> {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_config(mut self, config: lotabots_config::SecretsConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn with_metrics(mut self, metrics: M) -> Self {
        self.metrics = Some(metrics);
        self
    }
} 