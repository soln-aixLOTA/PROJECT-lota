use serde::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    // Server configuration
    pub port: u16,
    pub workers: Option<usize>,
    
    // Database configuration
    pub database_url: String,
    pub database_max_connections: u32,
    
    // Authentication configuration
    pub jwt_secret: String,
    pub jwt_expiration: i64,  // in minutes
    pub refresh_token_expiration: i64,  // in days
    
    // Security configuration
    pub password_hash_memory_cost: u32,
    pub password_hash_iterations: u32,
    pub max_login_attempts: u32,
    pub lockout_duration: i64,  // in minutes
    
    // Rate limiting
    pub rate_limit_window: u64,  // in seconds
    pub rate_limit_max_requests: u32,
    
    // Enterprise features
    pub mfa_required_tiers: Vec<String>,  // subscription tiers that require MFA
    pub min_password_length: usize,
    pub require_special_chars: bool,
    pub require_numbers: bool,
    pub require_uppercase: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            // Server configuration
            port: env::var("PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .expect("Invalid PORT"),
                
            workers: env::var("WORKERS")
                .ok()
                .map(|w| w.parse().expect("Invalid WORKERS")),
                
            // Database configuration
            database_url: env::var("DATABASE_URL")?,
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid DATABASE_MAX_CONNECTIONS"),
                
            // Authentication configuration
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "60".to_string())  // 60 minutes default
                .parse()
                .expect("Invalid JWT_EXPIRATION"),
                
            refresh_token_expiration: env::var("REFRESH_TOKEN_EXPIRATION")
                .unwrap_or_else(|_| "30".to_string())  // 30 days default
                .parse()
                .expect("Invalid REFRESH_TOKEN_EXPIRATION"),
                
            // Security configuration
            password_hash_memory_cost: env::var("PASSWORD_HASH_MEMORY_COST")
                .unwrap_or_else(|_| "65536".to_string())  // 64MB
                .parse()
                .expect("Invalid PASSWORD_HASH_MEMORY_COST"),
                
            password_hash_iterations: env::var("PASSWORD_HASH_ITERATIONS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .expect("Invalid PASSWORD_HASH_ITERATIONS"),
                
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid MAX_LOGIN_ATTEMPTS"),
                
            lockout_duration: env::var("LOCKOUT_DURATION")
                .unwrap_or_else(|_| "30".to_string())  // 30 minutes default
                .parse()
                .expect("Invalid LOCKOUT_DURATION"),
                
            // Rate limiting
            rate_limit_window: env::var("RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "3600".to_string())  // 1 hour default
                .parse()
                .expect("Invalid RATE_LIMIT_WINDOW"),
                
            rate_limit_max_requests: env::var("RATE_LIMIT_MAX_REQUESTS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .expect("Invalid RATE_LIMIT_MAX_REQUESTS"),
                
            // Enterprise features
            mfa_required_tiers: env::var("MFA_REQUIRED_TIERS")
                .unwrap_or_else(|_| "enterprise".to_string())
                .split(',')
                .map(String::from)
                .collect(),
                
            min_password_length: env::var("MIN_PASSWORD_LENGTH")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .expect("Invalid MIN_PASSWORD_LENGTH"),
                
            require_special_chars: env::var("REQUIRE_SPECIAL_CHARS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("Invalid REQUIRE_SPECIAL_CHARS"),
                
            require_numbers: env::var("REQUIRE_NUMBERS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("Invalid REQUIRE_NUMBERS"),
                
            require_uppercase: env::var("REQUIRE_UPPERCASE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .expect("Invalid REQUIRE_UPPERCASE"),
        })
    }
} 