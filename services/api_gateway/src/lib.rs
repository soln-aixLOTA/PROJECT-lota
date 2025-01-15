use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
    http::header,
    middleware::{Compress, Logger, NormalizePath},
    web, App, HttpServer,
};
use sqlx::migrate;
use std::env;
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

pub use error::ApiError;
pub use models::{ApiResponse, CreateUserRequest, LoginResponse, User, UserLogin};

pub async fn run() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load and validate required environment variables
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    // Ensure JWT_SECRET is set
    if env::var("JWT_SECRET").is_err() {
        eprintln!("JWT_SECRET environment variable must be set. Generate one with:");
        eprintln!("   openssl rand -hex 32");
        std::process::exit(1);
    }

    info!("Setting up database connection pool...");
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    info!("Running database migrations...");
    migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // Configure rate limiting
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2) // 2 requests per second
        .burst_size(5) // Allow bursts of 5 requests
        .finish()
        .unwrap();

    // Configure stricter rate limits for auth endpoints
    let auth_governor_conf = GovernorConfigBuilder::default()
        .per_second(1) // 1 request per second
        .burst_size(3) // Allow bursts of 3 requests
        .finish()
        .unwrap();

    info!("Starting server at http://{}", bind_address);

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin_fn(|origin, _req_head| {
                env::var("CORS_ORIGINS")
                    .map(|allowed| {
                        allowed.split(',').any(|o| {
                            let o = o.trim();
                            origin.as_bytes() == o.as_bytes()
                        })
                    })
                    .unwrap_or(false)
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .service(routes::health::health_check)
            .service(
                web::scope("/api/v1")
                    .wrap(Governor::new(&governor_conf))
                    .service(
                        web::scope("/auth")
                            .wrap(Governor::new(&auth_governor_conf))
                            .configure(routes::users::configure_auth),
                    )
                    .service(
                        web::scope("/users")
                            .wrap(middleware::auth::AuthMiddleware::new(
                                utils::get_jwt_secret().expect("JWT_SECRET validation failed"),
                            ))
                            .configure(routes::users::configure_users),
                    )
                    .service(
                        web::scope("/products")
                            .wrap(middleware::auth::AuthMiddleware::new(
                                utils::get_jwt_secret().expect("JWT_SECRET validation failed"),
                            ))
                            .configure(routes::products::configure),
                    ),
            )
    })
    .bind(bind_address)?
    .run()
    .await
}
