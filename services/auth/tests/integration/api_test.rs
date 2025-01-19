use actix_web::{test, web::Data, App};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

use lotabots_auth::{
    config::Config,
    handlers,
    repository::AuthRepository,
    service::AuthService,
    LoginRequest, RegisterRequest,
};

async fn setup_test_app() -> actix_web::App {
    // Load test config
    let config = Config {
        database_url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lotabots_test".to_string()),
        jwt_secret: "test_secret".to_string(),
        server_addr: "127.0.0.1:0".to_string(),
    };

    // Create database pool
    let pool = PgPoolOptions::new()
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

    // Create app
    App::new()
        .app_data(Data::new(service))
        .configure(handlers::config)
}

#[actix_web::test]
async fn test_register_endpoint() {
    let app = setup_test_app().await;
    let app = test::init_service(app).await;

    // Test successful registration
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["username"], "testuser");
    assert_eq!(body["email"], "test@example.com");

    // Test duplicate username
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "testuser",
            "email": "another@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    // Test invalid email
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "newuser",
            "email": "invalid-email",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_register_with_duplicate_email() {
    let app = setup_test_app().await;
    let app = test::init_service(app).await;

    // Register first user
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "user1",
            "email": "same@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Try to register second user with same email
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "user2",
            "email": "same@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Email already registered");
}

#[actix_web::test]
async fn test_username_case_sensitivity() {
    let app = setup_test_app().await;
    let app = test::init_service(app).await;

    // Register with uppercase username
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "TestUser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Try to register with same username in different case
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "testuser",
            "email": "another@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    // Try to login with original case
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "TestUser",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 200);

    // Try to login with different case
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "testuser",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_password_requirements() {
    let app = setup_test_app().await;
    let app = test::init_service(app).await;

    let test_cases = vec![
        (
            "short",
            "Password too short",
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "short"
            })
        ),
        (
            "no special chars",
            "Password must contain special characters",
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "password123"
            })
        ),
        (
            "no numbers",
            "Password must contain numbers",
            json!({
                "username": "testuser",
                "email": "test@example.com",
                "password": "passwordabc!"
            })
        ),
    ];

    for (case, expected_error, payload) in test_cases {
        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            400,
            "Expected 400 status for case: {}",
            case
        );

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains(expected_error),
            "Case '{}' failed: expected error containing '{}', got '{}'",
            case,
            expected_error,
            body["error"]
        );
    }
}

#[actix_web::test]
async fn test_login_endpoint() {
    let app = setup_test_app().await;
    let app = test::init_service(app).await;

    // First register a user
    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "logintest",
            "email": "login@example.com",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, register_req).await;
    assert_eq!(resp.status(), 201);

    // Test successful login
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "logintest",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(!body["token"].as_str().unwrap().is_empty());
    assert_eq!(body["username"], "logintest");

    // Test invalid credentials
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "logintest",
            "password": "wrongpassword"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 401);

    // Test non-existent user
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "nonexistent",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), 401);
}
