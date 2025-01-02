use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use deadpool_redis::Config as RedisConfig;
use middleware::{EnforceHttps, RateLimiter, SecurityHeaders};
use prometheus::Registry;
use tracing_actix_web::TracingLogger;

mod auth;
mod errors;
mod handlers;
mod middleware;
mod monitoring;
mod repositories;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables
    dotenv::dotenv().ok();

    // Initialize logging with OpenTelemetry
    init_telemetry();

    // Database connection setup
    let pool = setup_database().await?;

    // Redis connection setup
    let redis_pool = setup_redis()?;

    // Initialize Prometheus registry
    let prometheus_registry = Registry::new();

    // Get rate limit configuration from environment
    let rate_limit_requests = std::env::var("RATE_LIMIT_REQUESTS")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    let rate_limit_duration = std::env::var("RATE_LIMIT_DURATION")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);

    // Get allowed origins from environment
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "https://your-frontend-domain.com".to_string());

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&allowed_origins)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(EnforceHttps)
            .wrap(SecurityHeaders)
            .wrap(RateLimiter::new(
                redis_pool.clone(),
                rate_limit_requests,
                rate_limit_duration,
            ))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(prometheus_registry.clone()))
            // Health check endpoints
            .service(handlers::health::health_check)
            .service(handlers::health::health_status)
            .service(handlers::health::ssl_status)
            .service(handlers::health::domain_status)
            // Metrics endpoint
            .service(handlers::health::metrics)
            // API endpoints
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/refresh", web::post().to(handlers::auth::refresh_token))
                            .route("/validate", web::post().to(handlers::auth::validate_token)),
                    )
                    .service(
                        web::scope("/ai")
                            .route("/models", web::get().to(handlers::ai::list_models))
                            .route("/predict", web::post().to(handlers::ai::predict))
                            .route("/train", web::post().to(handlers::ai::train))
                            .route("/status/{job_id}", web::get().to(handlers::ai::job_status)),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

fn init_telemetry() {
    if std::env::var("TELEMETRY_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(opentelemetry::runtime::Tokio)
            .expect("Failed to initialize OpenTelemetry tracer");

        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_trace_id_width(32)
            .with_span_id_width(16)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }
}

async fn setup_database() -> std::io::Result<sqlx::PgPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connections = std::env::var("DATABASE_POOL_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

fn setup_redis() -> std::io::Result<deadpool_redis::Pool> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let pool_size = std::env::var("REDIS_POOL_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    let cfg = RedisConfig::from_url(redis_url);
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
