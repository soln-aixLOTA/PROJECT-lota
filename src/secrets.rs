use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::{error, info};
use vaultrs::{
    client::{VaultClient, VaultClientSettingsBuilder},
    error::ClientError,
};

use crate::config::AppConfig;

/// Represents a secret value with optional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    /// The actual secret value
    pub value: String,
    /// Additional metadata associated with the secret
    pub metadata: Option<serde_json::Value>,
}

/// A cached secret with expiration time
#[derive(Debug, Clone)]
struct CachedSecret {
    /// The secret value and metadata
    secret: Secret,
    /// When this cached value should expire
    expires_at: Instant,
}

/// Possible errors that can occur during secrets operations
#[derive(Debug)]
pub enum SecretsError {
    /// Vault client is not initialized
    VaultClientNotInitialized,
    /// Error from the Vault client
    VaultError(()),
    /// Error during serialization/deserialization
    SerializationError(String),
    /// HTTP client error
    HttpError(()),
}

impl From<ClientError> for SecretsError {
    fn from(_err: ClientError) -> Self {
        SecretsError::VaultError(())
    }
}

impl From<reqwest::Error> for SecretsError {
    fn from(_err: reqwest::Error) -> Self {
        SecretsError::HttpError(())
    }
}

/// Manages secrets using HashiCorp Vault with local caching
#[derive(Clone)]
pub struct SecretsManager {
    /// Vault client for direct API access
    client: Option<Arc<VaultClient>>,
    /// HTTP client for REST API calls
    http_client: Arc<Client>,
    /// Base URL of the Vault server
    base_url: String,
    /// Authentication token for Vault
    token: String,
    /// Local cache of secrets
    cache: Arc<DashMap<String, CachedSecret>>,
    /// Time-to-live for cached secrets
    ttl: Duration,
}

impl SecretsManager {
    /// Creates a new SecretsManager instance
    ///
    /// # Arguments
    /// * `config` - Application configuration containing Vault settings
    pub async fn new(config: &AppConfig) -> Self {
        let http_client = Client::new();
        let http_client = Arc::new(http_client);

        let (client, base_url, token) = if let Some(vault_config) = &config.vault_config {
            let token = vault_config.token.clone().unwrap_or_default();
            let settings = VaultClientSettingsBuilder::default()
                .address(&vault_config.address)
                .token(&token)
                .build()
                .expect("Failed to build Vault client settings");

            match VaultClient::new(settings) {
                Ok(client) => {
                    info!("Successfully initialized Vault client");
                    (Some(Arc::new(client)), vault_config.address.clone(), token)
                }
                Err(err) => {
                    error!("Failed to create Vault client: {}", err);
                    (None, String::new(), String::new())
                }
            }
        } else {
            info!("No Vault configuration provided, running without secrets management");
            (None, String::new(), String::new())
        };

        SecretsManager {
            client,
            http_client,
            base_url,
            token,
            cache: Arc::new(DashMap::new()),
            ttl: Duration::from_secs(300),
        }
    }

    /// Retrieves a secret from Vault or cache
    ///
    /// # Arguments
    /// * `path` - Path to the secret in Vault
    ///
    /// # Returns
    /// * `Result<Secret, SecretsError>` - The secret value or an error
    pub async fn get_secret(&self, path: &str) -> Result<Secret, SecretsError> {
        // Check cache first
        if let Some(cached) = self.cache.get(path) {
            if cached.expires_at > Instant::now() {
                info!("Cache hit for secret at path: {}", path);
                return Ok(cached.secret.clone());
            }
            info!("Cache expired for secret at path: {}", path);
            self.cache.remove(path);
        }

        // If no client, return error
        if self.client.is_none() {
            return Err(SecretsError::VaultClientNotInitialized);
        }

        info!("Fetching secret from Vault at path: {}", path);

        // Get from Vault using HTTP API
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        let response = self
            .http_client
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let secret = match response.get("data").and_then(|d| d.get("data")) {
            Some(value) => Secret {
                value: serde_json::to_string(value).unwrap_or_default(),
                metadata: None,
            },
            None => {
                error!("Invalid secret format received from Vault");
                return Err(SecretsError::SerializationError(
                    "Invalid secret format".to_string(),
                ));
            }
        };

        // Cache the result
        info!("Caching secret at path: {}", path);
        self.cache.insert(
            path.to_string(),
            CachedSecret {
                secret: secret.clone(),
                expires_at: Instant::now() + self.ttl,
            },
        );

        Ok(secret)
    }

    /// Stores a secret in Vault
    ///
    /// # Arguments
    /// * `path` - Path where to store the secret
    /// * `secret` - The secret to store
    ///
    /// # Returns
    /// * `Result<(), SecretsError>` - Success or error
    pub async fn set_secret(&self, path: &str, secret: Secret) -> Result<(), SecretsError> {
        // If no client, return error
        if self.client.is_none() {
            return Err(SecretsError::VaultClientNotInitialized);
        }

        info!("Setting secret at path: {}", path);

        let value: serde_json::Value = serde_json::from_str(&secret.value)
            .map_err(|e| SecretsError::SerializationError(e.to_string()))?;

        let payload = serde_json::json!({
            "data": {
                "data": value
            }
        });

        // Set in Vault using HTTP API
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        self.http_client
            .put(&url)
            .header("X-Vault-Token", &self.token)
            .json(&payload)
            .send()
            .await?;

        // Update cache
        info!("Updating cache for path: {}", path);
        self.cache.insert(
            path.to_string(),
            CachedSecret {
                secret: secret.clone(),
                expires_at: Instant::now() + self.ttl,
            },
        );

        Ok(())
    }

    /// Deletes a secret from Vault
    ///
    /// # Arguments
    /// * `path` - Path to the secret to delete
    ///
    /// # Returns
    /// * `Result<(), SecretsError>` - Success or error
    pub async fn delete_secret(&self, path: &str) -> Result<(), SecretsError> {
        // If no client, return error
        if self.client.is_none() {
            return Err(SecretsError::VaultClientNotInitialized);
        }

        info!("Deleting secret at path: {}", path);

        // Delete from Vault using HTTP API
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        self.http_client
            .delete(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        // Remove from cache
        info!("Removing from cache: {}", path);
        self.cache.remove(path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, VaultConfig};

    #[tokio::test]
    async fn test_secrets_manager_initialization() {
        let config = AppConfig {
            vault_config: Some(VaultConfig {
                address: "http://localhost:8200".to_string(),
                token: Some("test-token".to_string()),
                role_id: None,
                secret_id: None,
            }),
            ..AppConfig::default()
        };

        let manager = SecretsManager::new(&config).await;
        assert!(manager.client.is_some());
    }

    #[tokio::test]
    async fn test_secrets_manager_no_vault() {
        let config = AppConfig::default();
        let manager = SecretsManager::new(&config).await;
        assert!(manager.client.is_none());

        let result = manager.get_secret("test").await;
        assert!(matches!(
            result,
            Err(SecretsError::VaultClientNotInitialized)
        ));
    }
}
