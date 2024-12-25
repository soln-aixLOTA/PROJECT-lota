use thiserror::Error;
use serde::{Serialize, Deserialize};
use tracing::{error, warn, info};

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum LotaBotsError {
    #[error("Authentication failed: {message}")]
    Authentication { message: String, context: Option<serde_json::Value> },
    
    #[error("Authorization failed: {message}")]
    Authorization { message: String, context: Option<serde_json::Value> },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String, context: Option<serde_json::Value> },
    
    #[error("Validation error: {message}")]
    Validation { message: String, context: Option<serde_json::Value> },
    
    #[error("Internal error: {message}")]
    Internal { message: String, context: Option<serde_json::Value> },
}

impl LotaBotsError {
    pub fn log(&self) {
        match self {
            Self::Authentication { message, context } => {
                warn!(error_type = "authentication", %message, ?context, "Authentication error occurred");
            },
            Self::Internal { message, context } => {
                error!(error_type = "internal", %message, ?context, "Internal error occurred");
            },
            // ... other variants
        }
    }
}
