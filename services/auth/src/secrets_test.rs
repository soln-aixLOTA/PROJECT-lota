#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use tokio::test;

    // AWS Secrets Manager Tests
    mock! {
        pub AwsSecretManager {}

        #[async_trait]
        impl SecretManager for AwsSecretManager {
            async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
        }
    }

    #[tokio::test]
    async fn test_aws_get_secret_success() {
        let mut mock = MockAwsSecretManager::new();
        mock.expect_get_secret()
            .with(eq("my_secret"))
            .times(1)
            .returning(|_| Ok("test_secret_value".to_string()));

        let secret = mock.get_secret("my_secret").await.unwrap();
        assert_eq!(secret, "test_secret_value");
    }

    #[tokio::test]
    async fn test_aws_get_secret_not_found() {
        let mut mock = MockAwsSecretManager::new();
        mock.expect_get_secret()
            .with(eq("unknown_secret"))
            .times(1)
            .returning(|_| Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Secret not found",
            ))));

        let result = mock.get_secret("unknown_secret").await;
        assert!(result.is_err());
    }

    // Vault Secret Manager Tests
    mock! {
        pub VaultSecretManager {}

        #[async_trait]
        impl SecretManager for VaultSecretManager {
            async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
        }
    }

    #[tokio::test]
    async fn test_vault_get_secret_success() {
        let mut mock = MockVaultSecretManager::new();
        mock.expect_get_secret()
            .with(eq("path/to/secret#key"))
            .times(1)
            .returning(|_| Ok("vault_secret_value".to_string()));

        let secret = mock.get_secret("path/to/secret#key").await.unwrap();
        assert_eq!(secret, "vault_secret_value");
    }

    #[tokio::test]
    async fn test_vault_get_secret_invalid_format() {
        let mut mock = MockVaultSecretManager::new();
        mock.expect_get_secret()
            .with(eq("invalid_format"))
            .times(1)
            .returning(|_| Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid secret name format for Vault. Expected 'path/to/secret#key'",
            ))));

        let result = mock.get_secret("invalid_format").await;
        assert!(result.is_err());
    }

    // Azure Key Vault Tests
    mock! {
        pub AzureSecretManager {}

        #[async_trait]
        impl SecretManager for AzureSecretManager {
            async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
        }
    }

    #[tokio::test]
    async fn test_azure_get_secret_success() {
        let mut mock = MockAzureSecretManager::new();
        mock.expect_get_secret()
            .with(eq("azure_secret"))
            .times(1)
            .returning(|_| Ok("azure_secret_value".to_string()));

        let secret = mock.get_secret("azure_secret").await.unwrap();
        assert_eq!(secret, "azure_secret_value");
    }

    #[tokio::test]
    async fn test_azure_get_secret_auth_failure() {
        let mut mock = MockAzureSecretManager::new();
        mock.expect_get_secret()
            .with(eq("azure_secret"))
            .times(1)
            .returning(|_| Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Authentication failed",
            ))));

        let result = mock.get_secret("azure_secret").await;
        assert!(result.is_err());
    }

    // Google Cloud Secret Manager Tests
    mock! {
        pub GoogleSecretManager {
            pub project_id: String,
        }

        #[async_trait]
        impl SecretManager for GoogleSecretManager {
            async fn get_secret(&self, secret_name: &str) -> Result<String, Box<dyn std::error::Error>>;
        }
    }

    #[tokio::test]
    async fn test_google_get_secret_success() {
        let mut mock = MockGoogleSecretManager::new();
        mock.expect_get_secret()
            .with(eq("secret_id/version_id"))
            .times(1)
            .returning(|_| Ok("google_secret_value".to_string()));

        let secret = mock.get_secret("secret_id/version_id").await.unwrap();
        assert_eq!(secret, "google_secret_value");
    }

    #[tokio::test]
    async fn test_google_get_secret_invalid_format() {
        let mut mock = MockGoogleSecretManager::new();
        mock.expect_get_secret()
            .with(eq("invalid_format"))
            .times(1)
            .returning(|_| Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid secret name format for Google Cloud. Expected 'secret_id/version_id'",
            ))));

        let result = mock.get_secret("invalid_format").await;
        assert!(result.is_err());
    }
}
