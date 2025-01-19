use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    // Server settings
    pub host: String,
    pub port: u16,

    // Authentication settings
    pub jwt_secret: String,
    pub jwt_expiration: u64, // in seconds

    // Service registry
    pub service_urls: HashMap<String, String>,

    // Rate limiting settings
    pub rate_limit_requests: u32,
    pub rate_limit_duration: u32, // in seconds
}

impl AppConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        // Build configuration
        config::Config::builder()
            // Add configuration sources:
            // 1. Default values
            .set_default("host", "127.0.0.1")?
            .set_default("port", 8080)?
            .set_default("rate_limit_requests", 100)?
            .set_default("rate_limit_duration", 60)?
            // 2. Add configuration file
            .add_source(config::File::with_name("config").required(false))
            // 3. Add environment variables with prefix "LOTA_"
            .add_source(
                config::Environment::with_prefix("LOTA")
                    .separator("_")
                    .try_parsing(true)
            )
            .build()?
            .try_deserialize()
    }
}
