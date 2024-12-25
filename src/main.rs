use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod config;
mod error;
mod middleware;
mod models;
mod routes;
mod utils;

use config::Config;
use middleware::{
    auth::JwtAuth,
    audit_middleware::AuditMiddleware,
    metrics_middleware::Metrics,
    rate_limit_middleware::RateLimit,
    tenant_middleware::TenantMiddleware,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Config::load()?;
    let server_port = config.server_port;

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    let pool = Arc::new(pool);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Metrics::new())
            .wrap(RateLimit::new())
            .wrap(JwtAuth::new())
            .wrap(AuditMiddleware::new(pool.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api/v1")
                    .wrap(TenantMiddleware::new())
                    .service(routes::get_user)
            )
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await
}
