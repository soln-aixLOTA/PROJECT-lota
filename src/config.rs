use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub log_level: String,
    pub metrics_port: u16,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub vault: Option<VaultConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub address: String,
    pub token: Option<String>,
    pub role_id: Option<String>,
    pub secret_id: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            environment: "development".to_string(),
            log_level: "info".to_string(),
            metrics_port: 9090,
            rate_limit: RateLimitConfig {
                requests_per_second: 10,
                burst_size: 50,
            },
            cors: CorsConfig {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ],
                allowed_headers: vec!["*".to_string()],
                max_age: 3600,
            },
            vault: None,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // Start with default config
        s = s.set_default("host", "127.0.0.1")?;
        s = s.set_default("port", 8080)?;
        s = s.set_default("environment", "development")?;
        s = s.set_default("log_level", "info")?;
        s = s.set_default("metrics_port", 9090)?;

        // Add in config file if it exists
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            s = s.add_source(File::with_name(&config_path));
        }

        // Add in environment variables with prefix "APP_"
        s = s.add_source(Environment::with_prefix("APP").separator("_"));

        // Build config
        s.build()?.try_deserialize()
    }

    pub fn get_vault_config(&self) -> Option<VaultConfig> {
        self.vault.clone()
    }

    pub fn get_rate_limit_config(&self) -> RateLimitConfig {
        self.rate_limit.clone()
    }

    pub fn get_cors_config(&self) -> CorsConfig {
        self.cors.clone()
    }
}
