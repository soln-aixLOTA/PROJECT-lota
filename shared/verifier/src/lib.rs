use std::process::{Command, Stdio};
use std::time::Duration;
use std::io::Write;
use anyhow::{Result, anyhow};

mod code_verifier;
mod error;
mod models;
mod utils;

pub use code_verifier::{CodeVerifier, PythonVerifier};
pub use error::VerifierError;
pub use models::VerificationResult;

/// Constants for verification configuration
pub const TIMEOUT_SECONDS: u64 = 5;
pub const ALLOWED_IMPORTS: [&str; 4] = ["math", "sympy", "numpy", "scipy"];

/// Initialize the verifier module
pub fn init() -> Result<()> {
    tracing::info!("Initializing verifier module...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        assert!(init().is_ok());
    }
}
