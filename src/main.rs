use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use api_gateway::{
    config::AppConfig,
    handlers::auth::{get_current_user, login, register},
    middleware::auth::AuthMiddleware,
    middleware::cors::configure_cors,
    repositories::user::UserRepository,
    state::AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, sync::Arc};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create database connection pool
    let db_ssl_mode = match env::var("ENVIRONMENT").as_deref() {
        Ok("production") => "require",
        _ => "prefer",
    };
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5433/lotabots")
        .await
        .expect("Failed to connect to Postgres");

    // Initialize repositories
    let user_repo = UserRepository::new(pool.clone());

    // Initialize application state
    let state = web::Data::new(AppState::new(user_repo));

    info!("Starting server at http://127.0.0.1:8080");

    // Create HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(configure_cors())
            .wrap(AuthMiddleware::new("your-secret-key".to_string()))
            .app_data(state.clone())
            .service(
                web::scope("/api/v1")
                    .service(register)
                    .service(login)
                    .service(get_current_user),
            )
    })
    .bind("127.0.0.1:8080")?
    .workers(1)
    .run()
    .await
}
