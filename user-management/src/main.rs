use std::sync::Arc;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod events;
mod config;

use crate::{
    handlers::tenant_handlers,
    middleware::{
        audit_middleware::AuditMiddleware,
        metrics_middleware::MetricsMiddleware,
        rate_limit_middleware::RateLimitMiddleware,
        tenant_middleware::TenantMiddleware,
    },
    repositories::tenant_repository::PostgresTenantRepository,
    services::tenant_service::TenantService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize metrics registry
    prometheus_endpoint::init();

    // Initialize configuration
    let config = config::Config::load();

    // Initialize database connection pool
    let pool = database::create_pool(&config.database_url).await?;

    // Initialize event publisher
    let event_publisher = events::EventPublisher::new();

    // Initialize repositories and services
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));

    // Get server configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    tracing::info!("Starting server at http://{}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Enable logger
            .wrap(Logger::default())
            // Add services to app data
            .app_data(web::Data::new(tenant_service.clone()))
            .app_data(web::Data::new(event_publisher.clone()))
            // Add middleware in recommended order
            .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
            .wrap(MetricsMiddleware::new(tenant_service.clone()))
            .wrap(RateLimitMiddleware::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service.clone()))
            // Configure routes
            .service(
                web::scope("/api/v1")
                    .configure(tenant_handlers::configure)
                    // Add other handlers here
            )
            // Add Prometheus metrics endpoint
            .service(web::resource("/metrics").to(prometheus_endpoint::metrics))
            // Add health check endpoint
            .service(web::resource("/health").to(health_check))
    })
    .bind((host, port))?
    .run()
    .await
}

// Health check handler
async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// Prometheus metrics endpoint module
mod prometheus_endpoint {
    use actix_web::HttpResponse;
    use prometheus::{Encoder, TextEncoder};

    pub fn init() {
        // Initialize Prometheus registry (already done by middleware)
    }

    pub async fn metrics() -> HttpResponse {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::default_registry().gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        
        HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(buffer)
    }
} 