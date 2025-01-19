// HashiCorp Vault Implementation
pub struct VaultSecretManager;

#[async_trait]
impl SecretManager for VaultSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = secret_name.splitn(2, '#').collect();
        if parts.len() != 2 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid secret name format for Vault. Expected 'path/to/secret#key'",
            )));
        }
        let secret_path = parts[0];
        let secret_key = parts[1];
        get_secret_vault(secret_path, secret_key).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

// Azure Key Vault Implementation
pub struct AzureSecretManager;

#[async_trait]
impl SecretManager for AzureSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        get_secret_azure(secret_name).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

// Google Cloud Secret Manager Implementation
pub struct GoogleSecretManager {
    pub project_id: String,
}

#[async_trait]
impl SecretManager for GoogleSecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = secret_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid secret name format for Google Cloud. Expected 'secret_id/version_id'",
            )));
        }
        let secret_id = parts[0];
        let version_id = parts[1];
        get_secret_google_cloud(&self.project_id, secret_id, version_id).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

#[async_trait]
pub trait SecretManager {
    async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn create_secret(&self, secret_name: &str, secret_value: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn revoke_secret(&self, secret_name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
