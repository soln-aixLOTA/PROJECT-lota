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

<<<<<<< HEAD
pub mod api;
pub mod auth;
pub mod core;
pub mod db;
pub mod handlers;
pub mod models;
pub mod storage;
=======
pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod resource_management;
>>>>>>> 921251a (fetch)
