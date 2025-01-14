use actix_web::{test, web, App};
use document_automation::{
    handlers::auth,
    auth::jwt::JwtAuth,
    models::auth::{CreateUserRequest, LoginRequest, LoginResponse, UserRole},
};
use sqlx::PgPool;
use serde_json::json;
use std::{env, time::Duration};
use tokio::time::sleep;

async fn setup_test_db() -> PgPool {
    // Connect to the default postgres database
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5433/postgres")
        .await
        .expect("Failed to connect to Postgres");

    // Terminate existing connections and drop database
    sqlx::query(
        "SELECT pg_terminate_backend(pg_stat_activity.pid)
         FROM pg_stat_activity
         WHERE pg_stat_activity.datname = 'document_automation_test'
         AND pid <> pg_backend_pid()"
    )
    .execute(&pool)
    .await
    .expect("Failed to terminate connections");

    // Wait for connections to close
    sleep(Duration::from_secs(2)).await;

    // Drop database if exists
    sqlx::query("DROP DATABASE IF EXISTS document_automation_test")
        .execute(&pool)
        .await
        .expect("Failed to drop database");

    // Wait for drop to complete
    sleep(Duration::from_secs(2)).await;

    // Create fresh database
    sqlx::query("CREATE DATABASE document_automation_test")
        .execute(&pool)
        .await
        .expect("Failed to create test database");

    // Connect to test database
    let test_pool = PgPool::connect("postgres://postgres:postgres@localhost:5433/document_automation_test")
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&test_pool)
        .await
        .expect("Failed to run migrations");

    test_pool
}

// Add cleanup function
async fn cleanup_test_db(pool: &PgPool) {
    let _ = sqlx::query("TRUNCATE TABLE users CASCADE").execute(pool).await;
}

async fn setup_test_app(pool: PgPool) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .configure(auth::config)
    ).await
}

#[actix_web::test]
async fn test_jwt_authentication_flow() {
    env::set_var("JWT_SECRET", "test_secret_key_for_jwt_token_generation");
    let pool = setup_test_db().await;
    cleanup_test_db(&pool).await;
    let app = setup_test_app(pool.clone()).await;

    // Test user registration and token generation
    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123456",
            "role": "User"
        }))
        .to_request();

    let register_resp = test::call_service(&app, register_req).await;
    assert!(register_resp.status().is_success());

    let register_body: LoginResponse = test::read_body_json(register_resp).await;
    assert!(!register_body.access_token.is_empty());
    assert!(!register_body.refresh_token.is_empty());

    // Test token validation
    let validate_req = test::TestRequest::get()
        .uri("/auth/validate")
        .insert_header(("Authorization", format!("Bearer {}", register_body.access_token)))
        .to_request();

    let validate_resp = test::call_service(&app, validate_req).await;
    assert!(validate_resp.status().is_success());

    // Test invalid token
    let invalid_req = test::TestRequest::get()
        .uri("/auth/validate")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();

    let invalid_resp = test::call_service(&app, invalid_req).await;
    assert_eq!(invalid_resp.status().as_u16(), 401);

    // Test login with valid credentials
    let login_req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(json!({
            "username": "testuser",
            "password": "password123456"
        }))
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success());

    let login_body: LoginResponse = test::read_body_json(login_resp).await;
    assert!(!login_body.access_token.is_empty());
    assert!(!login_body.refresh_token.is_empty());

    // Test refresh token
    let refresh_req = test::TestRequest::post()
        .uri("/auth/refresh")
        .set_json(login_body.refresh_token)
        .to_request();

    let refresh_resp = test::call_service(&app, refresh_req).await;
    assert!(refresh_resp.status().is_success());

    let refresh_body: LoginResponse = test::read_body_json(refresh_resp).await;
    assert!(!refresh_body.access_token.is_empty());
    assert!(!refresh_body.refresh_token.is_empty());
}

#[actix_web::test]
async fn test_token_expiration() {
    // Override the token expiry for testing
    let jwt_auth = JwtAuth::new(b"test_secret");
    let token = jwt_auth.create_token("test_user", vec!["User".to_string()], 1)
        .expect("Failed to create token");

    // Wait for token to expire
    sleep(Duration::from_secs(2)).await;

    // Validate expired token
    let result = jwt_auth.validate_token(&token);
    assert!(result.is_err());
    assert!(format!("{:?}", result.unwrap_err()).contains("expired"));
}

#[actix_web::test]
async fn test_role_based_access() {
    env::set_var("JWT_SECRET", "test_secret_key_for_jwt_token_generation");
    let pool = setup_test_db().await;
    cleanup_test_db(&pool).await;
    let app = setup_test_app(pool.clone()).await;

    // Register admin user
    let admin_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "admin_user",
            "email": "admin@example.com",
            "password": "admin123456",
            "role": "Admin"
        }))
        .to_request();

    let admin_resp = test::call_service(&app, admin_req).await;
    assert!(admin_resp.status().is_success(), "Admin registration failed: {:?}", admin_resp.status());
    let admin_body: LoginResponse = test::read_body_json(admin_resp).await;

    // Register regular user
    let user_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "regular_user",
            "email": "user@example.com",
            "password": "user123456",
            "role": "User"
        }))
        .to_request();

    let user_resp = test::call_service(&app, user_req).await;
    assert!(user_resp.status().is_success(), "User registration failed: {:?}", user_resp.status());
    let user_body: LoginResponse = test::read_body_json(user_resp).await;

    // Validate admin token and check roles
    let admin_validate_req = test::TestRequest::get()
        .uri("/auth/validate")
        .insert_header(("Authorization", format!("Bearer {}", admin_body.access_token)))
        .to_request();

    let admin_validate_resp = test::call_service(&app, admin_validate_req).await;
    assert!(admin_validate_resp.status().is_success());
    let admin_validate_body: serde_json::Value = test::read_body_json(admin_validate_resp).await;
    assert!(admin_validate_body["roles"].as_array().unwrap().contains(&json!("admin")));

    // Validate user token and check roles
    let user_validate_req = test::TestRequest::get()
        .uri("/auth/validate")
        .insert_header(("Authorization", format!("Bearer {}", user_body.access_token)))
        .to_request();

    let user_validate_resp = test::call_service(&app, user_validate_req).await;
    assert!(user_validate_resp.status().is_success());
    let user_validate_body: serde_json::Value = test::read_body_json(user_validate_resp).await;
    assert!(user_validate_body["roles"].as_array().unwrap().contains(&json!("user")));
}
