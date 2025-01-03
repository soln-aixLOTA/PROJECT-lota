use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod error;
mod metrics;
mod middleware;
mod rate_limit;
mod routes;
mod tenant;
mod validation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .init();

    info!("Starting API Gateway service");

    // Initialize database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Initialize metrics
    let prometheus = prometheus::Registry::new();
    prometheus::default_registry()
        .register(Box::new(metrics::HTTP_REQUESTS_TOTAL.clone()))
        .expect("Failed to register metric");
    prometheus::default_registry()
        .register(Box::new(metrics::HTTP_REQUEST_DURATION_SECONDS.clone()))
        .expect("Failed to register metric");

    // Get server configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    info!("Starting server at http://{}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // Enable logger middleware
            .wrap(Logger::default())
            // Enable metrics middleware
            .wrap(metrics::MetricsMiddleware::new())
            // Enable rate limiting middleware
            .wrap(rate_limit::RateLimitMiddleware::new())
            // Enable tenant middleware
            .wrap(tenant::TenantMiddleware::new(db_pool.clone()))
            // Enable validation middleware
            .wrap(validation::ValidationMiddleware::new())
            // Register routes
            .configure(routes::configure)
            // Add database pool to app state
            .app_data(web::Data::new(db_pool.clone()))
            // Add Prometheus registry to app state
            .app_data(web::Data::new(prometheus.clone()))
    })
    .bind((host, port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_server_initialization() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database pool");

        let app = test::init_service(
            App::new()
                .wrap(Logger::default())
                .wrap(metrics::MetricsMiddleware::new())
                .wrap(rate_limit::RateLimitMiddleware::new())
                .wrap(tenant::TenantMiddleware::new(db_pool.clone()))
                .wrap(validation::ValidationMiddleware::new())
                .configure(routes::configure)
                .app_data(web::Data::new(db_pool.clone())),
        )
        .await;

        assert!(app.is_ok());
    }
}
