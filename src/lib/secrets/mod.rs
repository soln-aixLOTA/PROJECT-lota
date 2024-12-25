use std::sync::Arc;
use tokio::sync::Mutex;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::error::ClientError;

#[derive(Clone)]
pub struct SecretsManager {
    client: Arc<Mutex<VaultClient>>,
}

impl SecretsManager {
    pub async fn new() -> Result<Self, ClientError> {
        let vault_addr = std::env::var("VAULT_ADDR")
            .unwrap_or_else(|_| "http://vault.lotabots.svc:8200".to_string());
        
        let vault_token = std::env::var("VAULT_TOKEN")
            .expect("VAULT_TOKEN must be set");

        let client = VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(&vault_addr)
                .token(&vault_token)
                .build()?
        )?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub async fn get_secret(&self, path: &str, key: &str) -> Result<String, ClientError> {
        let client = self.client.lock().await;
        let secret = client
            .read_secret_version(vaultrs::api::secrets::kv::v2::requests::ReadSecretVersionRequest {
                mount_point: "secret",
                path,
                version: None,
            })
            .await?;

        Ok(secret.data.data.get(key)
            .ok_or_else(|| ClientError::Other(format!("Key '{}' not found in secret", key)))?
            .as_str()
            .ok_or_else(|| ClientError::Other("Secret value is not a string".to_string()))?
            .to_string())
    }

    pub async fn set_secret(&self, path: &str, key: &str, value: &str) -> Result<(), ClientError> {
        let client = self.client.lock().await;
        let mut data = std::collections::HashMap::new();
        data.insert(key.to_string(), value.to_string());

        client
            .write_secret_version(vaultrs::api::secrets::kv::v2::requests::WriteSecretVersionRequest {
                mount_point: "secret",
                path,
                data,
            })
            .await?;

        Ok(())
    }
} 