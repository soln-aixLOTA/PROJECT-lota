use actix_web::{middleware, web::Data, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use lotabots_auth::{
    config::Config,
    handlers,
    middleware::rate_limit::RateLimiter,
    repository::AuthRepository,
    service::AuthService,
};
use shared::config::SecretsManager;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load config
    let config = Config::from_env().expect("Failed to load configuration");
    let addr = config.server_addr.clone();

    // Create database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Create repository and service
    let repository = AuthRepository::new(pool);
    let jwt_secret = SecretsManager::get_secret("JWT_SECRET")
        .expect("JWT_SECRET must be set");
    let service = AuthService::new(repository, jwt_secret);

    info!("Starting server at {}", addr);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(RateLimiter::new(60)) // 60 requests per minute
            .wrap(middleware::Logger::default())
            .app_data(Data::new(service.clone()))
            .configure(handlers::config)
    })
    .bind(addr)?
    .run()
    .await
}
