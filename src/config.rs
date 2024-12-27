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
    /// The deployment environment (development, staging, production)
    pub environment: String,
    /// Global log level for the service
    pub log_level: String,
    /// Port for exposing metrics (e.g., Prometheus)
    pub metrics_port: u16,
    /// HashiCorp Vault configuration for secrets management
    pub vault_config: Option<VaultConfig>,
    /// Rate limiting configuration to prevent abuse
    pub rate_limit: RateLimitConfig,
    /// CORS configuration for API access control
    pub cors: CorsConfig,
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
    /// Where to output logs (stdout, file, etc.)
    pub output: String,
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
            vault_config: None,
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "text".to_string(),
                output: "stdout".to_string(),
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

    /// Retrieves the Vault configuration if configured
    pub fn get_vault_config(&self) -> Option<VaultConfig> {
        self.vault_config.clone()
    }

    /// Retrieves the rate limiting configuration
    pub fn get_rate_limit_config(&self) -> RateLimitConfig {
        self.rate_limit.clone()
    }

    /// Retrieves the CORS configuration
    pub fn get_cors_config(&self) -> CorsConfig {
        self.cors.clone()
    }
}
