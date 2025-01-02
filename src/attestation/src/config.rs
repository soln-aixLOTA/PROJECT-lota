use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub log: LogConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("server.workers", i64::from(num_cpus::get() as u32))?
            .set_default("database.max_connections", 5)?
            .set_default("log.level", "info")?;

        // Add config file if it exists
        if let Ok(config_path) = std::env::var("CONFIG_FILE") {
            builder = builder.add_source(File::with_name(&config_path));
        }

        // Add environment variables
        builder = builder.add_source(
            Environment::with_prefix("LOTA")
                .separator("_")
                .try_parsing(true),
        );

        builder.build()?.try_deserialize()
    }
}
