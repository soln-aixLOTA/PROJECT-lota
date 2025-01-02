use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for the API Gateway service.
/// This structure contains all the configuration parameters needed for the service to run,
/// including infrastructure settings, security policies, and operational parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// The host address to bind the service to
    pub host: String,
    /// The port number to listen on
    pub port: u16,
    /// HashiCorp Vault configuration for secrets management
    pub vault_config: Option<VaultConfig>,
    /// Logging configuration for the service
    pub logging: LoggingConfig,
}

/// Configuration for the logging system.
/// Controls how logs are formatted and where they are output.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json or text)
    pub format: String,
}

/// Configuration for rate limiting.
/// Used to prevent API abuse by limiting request frequency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per second
    pub requests_per_second: u32,
    /// Maximum burst size for request spikes
    pub burst_size: u32,
}

/// Cross-Origin Resource Sharing (CORS) configuration.
/// Controls which domains can access the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// List of allowed origin domains
    pub allowed_origins: Vec<String>,
    /// List of allowed HTTP methods
    pub allowed_methods: Vec<String>,
    /// List of allowed HTTP headers
    pub allowed_headers: Vec<String>,
    /// How long the browser should cache CORS response
    pub max_age: u32,
}

/// HashiCorp Vault configuration for secrets management.
/// Used to securely store and retrieve sensitive information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Vault server address
    pub address: String,
    /// Authentication token for Vault
    pub token: Option<String>,
    /// Role ID for AppRole authentication
    pub role_id: Option<String>,
    /// Secret ID for AppRole authentication
    pub secret_id: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            vault_config: None,
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "text".to_string(),
            },
        }
    }
}

impl AppConfig {
    /// Creates a new AppConfig instance from environment variables and configuration files.
    ///
    /// # Environment Variables
    /// - CONFIG_PATH: Path to the configuration file
    /// - APP_*: Environment variables with APP_ prefix override configuration values
    ///
    /// # Returns
    /// - Result<AppConfig, ConfigError>: The loaded configuration or an error
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // Start with default config
        s = s.set_default("host", "127.0.0.1")?;
        s = s.set_default("port", 8080)?;

        // Add in config file if it exists
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            s = s.add_source(File::with_name(&config_path));
        }

        // Add in environment variables with prefix "APP_"
        s = s.add_source(Environment::with_prefix("APP").separator("_"));

        // Build config
        s.build()?.try_deserialize()
    }
}
