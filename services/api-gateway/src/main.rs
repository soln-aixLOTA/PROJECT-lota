use actix_web::{web, App, HttpServer, middleware};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;

mod error;
mod config;
mod routes;
mod metrics;
mod rate_limit;
mod usage_tracking;
mod tenant;
mod secrets;
mod worker_pool;
mod validation;
mod logging;

use config::AppConfig;
use error::ApiError;
use logging::{Logger, LoggerMiddleware, init_logging};
use metrics::init_metrics;
use rate_limit::{RateLimitConfig, RateLimitMiddleware};
use usage_tracking::UsageTracker;
use tenant::TenantMiddleware;
use secrets::{SecretsManager, SecretConfig};
use validation::{ValidationMiddleware, ValidationConfig};
use worker_pool::{AdaptiveWorkerPool, Request, Response};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    init_logging();
    info!("Starting API Gateway");

    // Load configuration
    let config = Arc::new(AppConfig::load().expect("Failed to load configuration"));
    info!("Configuration loaded successfully");

    // Initialize metrics
    init_metrics();
    info!("Metrics initialized");

    // Initialize secrets manager
    let secrets = Arc::new(SecretsManager::new(
        SecretConfig::load().expect("Failed to load secrets configuration")
    ));
    info!("Secrets manager initialized");

    // Initialize worker pool
    let worker_pool = Arc::new(AdaptiveWorkerPool::new(Arc::clone(&config)).await);
    info!("Worker pool initialized with {} workers", config.min_workers);

    // Start worker pool health monitoring
    let worker_pool_monitor = Arc::clone(&worker_pool);
    tokio::spawn(async move {
        worker_pool_monitor.monitor_health().await;
    });

    // Initialize shared components
    let usage_tracker = Arc::new(UsageTracker::new());
    let logger = Arc::new(Logger::new());
    let rate_limit_config = Arc::new(RateLimitConfig::new(config.rate_limit_requests_per_second));
    let validation_config = Arc::new(ValidationConfig::new());

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Add state
            .app_data(web::Data::from(Arc::clone(&config)))
            .app_data(web::Data::from(Arc::clone(&secrets)))
            .app_data(web::Data::from(Arc::clone(&worker_pool)))
            .app_data(web::Data::from(Arc::clone(&usage_tracker)))
            .app_data(web::Data::from(Arc::clone(&logger)))
            
            // Add middleware in the correct order
            .wrap(middleware::Logger::default())
            .wrap(LoggerMiddleware::new(Arc::clone(&logger)))
            .wrap(TenantMiddleware::new())
            .wrap(RateLimitMiddleware::new(Arc::clone(&rate_limit_config)))
            .wrap(ValidationMiddleware::new(Arc::clone(&validation_config)))
            
            // Configure routes with proper error handling
            .configure(routes::configure)
            
            // Add default error handler
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                ApiError::new(
                    "INVALID_JSON",
                    &format!("Invalid JSON payload: {}", err),
                    "system",
                )
                .into()
            }))
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .workers(config.num_workers)
    .run()
    .await
}