use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lota_user_management::{config::Config, handlers, repositories::UserRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::new().expect("Failed to load configuration");

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Create user repository
    let user_repository = web::Data::new(UserRepository::new(pool));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(user_repository.clone())
            .configure(handlers::configure_routes)
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .workers(config.server.workers)
    .run()
    .await
}
