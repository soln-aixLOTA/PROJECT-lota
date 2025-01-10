#![feature(portable_simd)]

//! Common utilities and types shared across services

pub mod error;
pub mod logging;
pub mod middleware;
pub mod stree;

// Re-export common types
pub use error::Error;
pub use logging::init_logging;
