use actix_web::{web, App, HttpServer};
use tokio::sync::RwLock;
use std::sync::Arc;
use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod state;

use crate::config::Config;
use crate::error::Error;
use crate::middleware::{auth::AuthMiddleware, rate_limit::RateLimitMiddleware};
use crate::state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize configuration
    let config = Config::from_env().expect("Failed to load configuration");
    
    // Initialize tracing
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("api-gateway")
        .install_simple()
        .expect("Failed to install Jaeger tracer");
    
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Initialize shared state
    let state = Arc::new(AppState::new(config.clone()).await?);
    
    // Create HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(AuthMiddleware::new(state.clone()))
            .wrap(RateLimitMiddleware::new(state.clone()))
            .configure(handlers::configure_routes)
    })
    .bind(format!("0.0.0.0:{}", config.port))?
    .workers(config.workers)
    .backlog(config.backlog)
    .keep_alive(config.keep_alive)
    .client_timeout(config.client_timeout)
    .client_shutdown(config.client_shutdown)
    .shutdown_timeout(config.shutdown_timeout)
    .max_connection_rate(config.max_connection_rate)
    .max_connections(config.max_connections);
    
    // Start server
    println!("Starting server at http://0.0.0.0:{}", config.port);
    let result = server.run().await;
    
    // Shutdown tracing
    global::shutdown_tracer_provider();
    
    result
} 