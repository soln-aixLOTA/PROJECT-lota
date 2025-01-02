use config::{Config as ConfigLoader, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub environment: String,
    pub log_level: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config".to_string());

        let environment =
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config = ConfigLoader::builder()
            // Start with default values
            .set_default("host", "127.0.0.1")?
            .set_default("port", 8081)?
            .set_default("environment", "development")?
            .set_default("log_level", "info")?
            // Add config file
            .add_source(File::from(PathBuf::from(&config_path).join("base.yaml")))
            // Add environment-specific config file
            .add_source(
                File::from(PathBuf::from(&config_path).join(format!("{}.yaml", environment)))
                    .required(false),
            )
            // Add environment variables (e.g., `USER_MANAGEMENT_PORT`)
            .add_source(Environment::with_prefix("USER_MANAGEMENT").separator("_"))
            .build()?;

        config.try_deserialize().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        std::env::set_var("CONFIG_PATH", "tests/config");
        let config = Config::load().unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8081);
        assert_eq!(config.environment, "development");
        assert_eq!(config.log_level, "info");
    }
}
