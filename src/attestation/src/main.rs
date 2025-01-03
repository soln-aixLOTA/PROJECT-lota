use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use lota_attestation::{config::Config, handlers, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = Config::new().expect("Failed to load configuration");

    // Set up logging
    std::env::set_var("RUST_LOG", &config.log.level);
    tracing_subscriber::fmt::init();

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Create app state
    let state = AppState::new(pool, config.clone());

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .app_data(Data::new(state.clone()))
            .configure(handlers::configure)
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run()
    .await
}
