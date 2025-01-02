use std::sync::Arc;
use tokio::sync::RwLock;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::auth::kubernetes::KubernetesAuth;
use vaultrs::error::ClientError;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn};
use tokio::time::{Duration, Instant};

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("Vault error: {0}")]
    VaultError(#[from] ClientError),
    
    #[error("Secret not found: {0}")]
    NotFound(String),
    
    #[error("Secret access denied: {0}")]
    AccessDenied(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Lease expired: {0}")]
    LeaseExpired(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretConfig {
    vault_addr: String,
    vault_role: String,
    namespace: String,
}

#[derive(Clone, Debug)]
struct CachedSecret {
    value: String,
    lease_id: Option<String>,
    expiry: Instant,
}

pub struct SecretsManager {
    client: Arc<VaultClient>,
    cache: Arc<RwLock<lru::LruCache<String, CachedSecret>>>,
    config: SecretConfig,
    lease_manager: Arc<LeaseManager>,
}

pub struct LeaseManager {
    client: Arc<VaultClient>,
    leases: Arc<RwLock<std::collections::HashMap<String, Duration>>>,
}

impl LeaseManager {
    pub fn new(client: Arc<VaultClient>) -> Self {
        let leases = Arc::new(RwLock::new(std::collections::HashMap::new()));
        let manager = Self { client, leases };
        
        // Spawn lease renewal task
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.run_lease_renewal().await;
        });
        
        manager
    }
    
    pub async fn register_lease(&self, lease_id: String, ttl: Duration) {
        self.leases.write().await.insert(lease_id, ttl);
    }
    
    async fn run_lease_renewal(self) {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            
            let leases = self.leases.read().await.clone();
            for (lease_id, ttl) in leases {
                if let Err(e) = self.client.sys().renew_lease(&lease_id, None).await {
                    error!("Failed to renew lease {}: {:?}", lease_id, e);
                    self.leases.write().await.remove(&lease_id);
                }
            }
        }
    }
}

impl Clone for LeaseManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            leases: self.leases.clone(),
        }
    }
}

impl SecretsManager {
    pub async fn new(config: SecretConfig) -> Result<Self, SecretError> {
        // Read Kubernetes service account token
        let k8s_token = tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
            .await
            .map_err(|e| SecretError::SystemError(format!("Failed to read k8s token: {}", e)))?;
            
        // Initialize Vault client with Kubernetes auth
        let auth = KubernetesAuth::new(&config.vault_role, &k8s_token);
        
        let client = VaultClientSettingsBuilder::default()
            .address(&config.vault_addr)
            .auth(auth)
            .build()
            .map(VaultClient::new)
            .map_err(|e| SecretError::SystemError(e.to_string()))?;
            
        let client = Arc::new(client);
        let lease_manager = Arc::new(LeaseManager::new(client.clone()));

        Ok(Self {
            client,
            cache: Arc::new(RwLock::new(lru::LruCache::new(100))),
            config,
            lease_manager,
        })
    }

    pub async fn get_secret(&self, key: &str) -> Result<String, SecretError> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.get(key) {
            if Instant::now() < cached.expiry {
                return Ok(cached.value.clone());
            }
        }

        // Fetch from Vault
        let path = format!("{}/{}", self.config.namespace, key);
        let secret = self.client
            .read_secret(&path)
            .await
            .map_err(|e| match e {
                ClientError::NotFound => SecretError::NotFound(key.to_string()),
                ClientError::Forbidden => SecretError::AccessDenied(key.to_string()),
                _ => SecretError::VaultError(e),
            })?;
            
        let lease_id = secret.lease_id.clone();
        let ttl = Duration::from_secs(secret.lease_duration);
        
        // Cache the result
        let cached_secret = CachedSecret {
            value: secret.data.clone(),
            lease_id: lease_id.clone(),
            expiry: Instant::now() + ttl,
        };
        
        self.cache.write().await.put(key.to_string(), cached_secret);
        
        // Register lease if present
        if let Some(lease_id) = lease_id {
            self.lease_manager.register_lease(lease_id, ttl).await;
        }

        Ok(secret.data)
    }

    pub async fn get_dynamic_secret(&self, path: &str) -> Result<String, SecretError> {
        // Dynamic secrets are never cached
        let secret = self.client
            .read_dynamic_secret(&format!("{}/{}", self.config.namespace, path))
            .await
            .map_err(|e| match e {
                ClientError::NotFound => SecretError::NotFound(path.to_string()),
                ClientError::Forbidden => SecretError::AccessDenied(path.to_string()),
                _ => SecretError::VaultError(e),
            })?;
            
        // Register lease for renewal
        if let Some(lease_id) = &secret.lease_id {
            self.lease_manager
                .register_lease(
                    lease_id.clone(),
                    Duration::from_secs(secret.lease_duration)
                )
                .await;
        }
        
        Ok(secret.data)
    }

    pub async fn set_secret(&self, key: &str, value: &str) -> Result<(), SecretError> {
        let path = format!("{}/{}", self.config.namespace, key);
        self.client
            .write_secret(&path, value)
            .await
            .map_err(SecretError::VaultError)?;

        // Update cache
        let cached_secret = CachedSecret {
            value: value.to_string(),
            lease_id: None,
            expiry: Instant::now() + Duration::from_secs(3600), // 1 hour cache for static secrets
        };
        self.cache.write().await.put(key.to_string(), cached_secret);

        Ok(())
    }

    pub async fn delete_secret(&self, key: &str) -> Result<(), SecretError> {
        let path = format!("{}/{}", self.config.namespace, key);
        self.client
            .delete_secret(&path)
            .await
            .map_err(SecretError::VaultError)?;

        // Remove from cache
        self.cache.write().await.pop(key);

        Ok(())
    }
} 