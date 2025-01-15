use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String, // Loaded from environment variable
    pub server_addr: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
        })
    }
}
