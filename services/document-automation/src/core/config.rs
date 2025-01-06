use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub ocr: OCRConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub bucket_name: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiration_hours: i64,
    pub encryption_key: String,
}

#[derive(Debug, Deserialize)]
pub struct OCRConfig {
    pub tesseract_data_path: String,
    pub supported_languages: Vec<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    S3,
    GCS,
    Local,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start with default settings
            .add_source(File::with_name("config/default"))
            // Add environment-specific settings
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local settings
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP_"
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?;

        s.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: num_cpus::get(),
            },
            database: DatabaseConfig {
                url: "postgres://postgres:postgres@localhost/docautomation".to_string(),
                max_connections: 5,
                idle_timeout_seconds: 300,
            },
            storage: StorageConfig {
                provider: StorageProvider::Local,
                bucket_name: "documents".to_string(),
                region: "us-east-1".to_string(),
                endpoint: None,
                access_key_id: "default".to_string(),
                secret_access_key: "default".to_string(),
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key".to_string(),
                token_expiration_hours: 24,
                encryption_key: "your-encryption-key".to_string(),
            },
            ocr: OCRConfig {
                tesseract_data_path: "/usr/share/tesseract-ocr/4.00/tessdata".to_string(),
                supported_languages: vec!["eng".to_string()],
                timeout_seconds: 30,
            },
        }
    }
}
