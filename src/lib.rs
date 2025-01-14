//! # LotaBots API Gateway
//!
//! The API Gateway serves as the main entry point for all client requests,
//! handling authentication, routing, and rate limiting.
//!
//! ## Modules
//!
//! - `handlers`: Contains route handlers for different API endpoints
//! - `middleware`: Implements cross-cutting concerns like authentication
//! - `errors`: Defines the error handling system
//! - `config`: Manages application configuration
//!
//! ## Example
//!
//! ```rust
//! use lotabots_api_gateway::App;
//!
//! #[actix_web::main]
//! async fn main() {
//!     App::new().run().await.unwrap();
//! }
//! ```

pub mod auth;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;

pub use error::{AppError, AppResult};

