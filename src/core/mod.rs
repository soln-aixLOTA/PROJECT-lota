use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod error;

pub struct AppState {
    pub db: PgPool,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn default_dev() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "dev_user".to_string(),
            password: "dev_password".to_string(),
            database: "document_automation_dev".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: String,
}

impl StorageConfig {
    pub fn default_dev() -> Self {
        Self {
            base_path: "storage".to_string(),
        }
    }
}
