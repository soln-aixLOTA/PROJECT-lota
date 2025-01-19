use actix_web::{test, web::Data, App};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

use lotabots_auth::{
    config::Config,
    handlers,
    middleware::rate_limit::RateLimiter,
    repository::AuthRepository,
    service::AuthService,
};

async fn setup_rate_limited_app() -> actix_web::App {
    // Load test config
    let config = Config {
        database_url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lotabots_test".to_string()),
        jwt_secret: "test_secret".to_string(),
        server_addr: "127.0.0.1:0".to_string(),
    };

    // Create database pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create repository and service
    let repository = AuthRepository::new(pool);
    let service = AuthService::new(repository, config.jwt_secret);

    // Create app with rate limiting
    App::new()
        .wrap(RateLimiter::new(30)) // 30 requests per minute
        .app_data(Data::new(service))
        .configure(handlers::config)
}

#[actix_web::test]
async fn test_rate_limiting_register() {
    let app = setup_rate_limited_app().await;
    let app = test::init_service(app).await;

    // Send requests up to the limit
    for i in 0..30 {
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(json!({
                "username": format!("user{}", i),
                "email": format!("user{}@example.com", i),
                "password": "Password123!@#"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            201,
            "Request {} should succeed",
            i
        );
    }

    // Next request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "overlimit",
            "email": "overlimit@example.com",
            "password": "Password123!@#"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);

    // Wait for rate limit to reset
    sleep(Duration::from_secs(2)).await;

    // Should be able to make request again
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "afterwait",
            "email": "afterwait@example.com",
            "password": "Password123!@#"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
}

#[actix_web::test]
async fn test_rate_limiting_login() {
    let app = setup_rate_limited_app().await;
    let app = test::init_service(app).await;

    // First register a test user
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "ratelimituser",
            "email": "ratelimit@example.com",
            "password": "Password123!@#"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Send login requests up to the limit
    for i in 0..30 {
        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(json!({
                "username": "ratelimituser",
                "password": "Password123!@#"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Login request {} should succeed",
            i
        );
    }

    // Next request should be rate limited
    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "ratelimituser",
            "password": "Password123!@#"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);

    // Test rate limit headers
    assert!(resp.headers().contains_key("retry-after"));
}

#[actix_web::test]
async fn test_rate_limiting_per_ip() {
    let app = setup_rate_limited_app().await;
    let app = test::init_service(app).await;

    // Send requests from different IPs
    for i in 0..2 {
        let ip = format!("192.168.1.{}", i);

        // Each IP should be able to make 30 requests
        for j in 0..30 {
            let req = test::TestRequest::post()
                .uri("/auth/register")
                .insert_header(("x-forwarded-for", &ip))
                .set_json(json!({
                    "username": format!("user{}{}", i, j),
                    "email": format!("user{}{}@example.com", i, j),
                    "password": "Password123!@#"
                }))
                .to_request();

            let resp = test::call_service(&app, req).await;
            assert_eq!(
                resp.status(),
                201,
                "Request {} from IP {} should succeed",
                j,
                ip
            );
        }

        // 31st request from each IP should be rate limited
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .insert_header(("x-forwarded-for", &ip))
            .set_json(json!({
                "username": format!("overlimit{}", i),
                "email": format!("overlimit{}@example.com", i),
                "password": "Password123!@#"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            429,
            "Request 31 from IP {} should be rate limited",
            ip
        );
    }
}
