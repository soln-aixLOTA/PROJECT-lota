use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL").map_err(|_| {
                config::ConfigError::NotFound("DATABASE_URL".to_string())
            })?,
            redis_url: env::var("REDIS_URL").map_err(|_| {
                config::ConfigError::NotFound("REDIS_URL".to_string())
            })?,
            jwt_secret: env::var("JWT_SECRET").map_err(|_| {
                config::ConfigError::NotFound("JWT_SECRET".to_string())
            })?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        env::set_var("HOST", "localhost");
        env::set_var("PORT", "8080");
        env::set_var("DATABASE_URL", "postgres://user:pass@localhost/db");
        env::set_var("REDIS_URL", "redis://localhost");
        env::set_var("JWT_SECRET", "test_secret");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.jwt_secret, "test_secret");
    }
}
