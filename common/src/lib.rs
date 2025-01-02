// This file will serve as the entry point for the common library

pub mod auth;
pub mod middleware;
pub mod types;
pub mod config;
pub mod error;
pub mod logging;

pub use auth::*;
pub use middleware::*;
pub use types::*;
pub use config::*;
pub use error::*;
pub use logging::*;

/// Initialize the common library components
pub async fn init() -> anyhow::Result<()> {
    tracing::info!("Initializing common library components");
    
    // Initialize logging
    logging::init()?;
    
    // Initialize other common components as needed
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        assert!(init().await.is_ok());
    }
}
