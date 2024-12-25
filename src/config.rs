use std::env;

#[derive(Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn load() -> Result<Self, std::io::Error> {
        Ok(Config {
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "SERVER_PORT must be a number"))?,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "DATABASE_URL must be set"))?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "JWT_SECRET must be set"))?,
        })
    }
}
