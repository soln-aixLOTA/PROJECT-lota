use actix_web::{
    test,
    web,
    App,
};
use document_automation::{
    handlers::{auth, documents},
};
use sqlx::PgPool;
use serde_json::json;
use std::env;

async fn cleanup_database(pool: &PgPool) {
    sqlx::query!("DELETE FROM users WHERE username = $1", "testuser")
        .execute(pool)
        .await
        .expect("Failed to clean up test user");
}

async fn setup_db() -> PgPool {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5433/postgres")
        .await
        .expect("Failed to connect to Postgres");

    // Drop and recreate the test database
    sqlx::query!("DROP DATABASE IF EXISTS document_automation_test")
        .execute(&pool)
        .await
        .expect("Failed to drop test database");

    sqlx::query!("CREATE DATABASE document_automation_test")
        .execute(&pool)
        .await
        .expect("Failed to create test database");

    // Connect to the test database
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

async fn test_app(pool: PgPool) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .wrap(document_automation::middleware::rate_limit::RateLimiter::new(100.0))
            .configure(auth::config)
            .service(
                web::scope("/api/v1")
                    .service(documents::create_document)
                    .service(documents::get_document)
                    .service(documents::list_documents)
            )
    ).await
}

#[actix_web::test]
async fn test_auth_flow() {
    env::set_var("JWT_SECRET", "test_secret_key_for_jwt_token_generation");

    let pool = setup_db().await;

    let app = test_app(pool.clone()).await;

    // Register a test user
    let register_req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123",
            "role": "User"
        }))
        .to_request();

    let register_resp = test::call_service(&app, register_req).await;
    let status = register_resp.status();
    println!("Register request: Status = {}", status);
    let register_body: serde_json::Value = if status.is_success() {
        test::read_body_json(register_resp).await
    } else {
        let body = test::read_body(register_resp).await;
        println!("Register response body: {:?}", String::from_utf8_lossy(&body));
        panic!("Registration failed with status: {}", status);
    };

    let access_token = register_body["access_token"].as_str().unwrap();

    // Test accessing protected route with token
    let protected_req = test::TestRequest::get()
        .uri("/api/v1/documents")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let protected_resp = test::call_service(&app, protected_req).await;
    let status = protected_resp.status();
    println!("Protected request: Status = {}", status);
    if !status.is_success() {
        let body = test::read_body(protected_resp).await;
        println!("Protected response body: {:?}", String::from_utf8_lossy(&body));
    }
    assert!(status.is_success());

    // Test accessing protected route without token
    let unauth_req = test::TestRequest::get()
        .uri("/api/v1/documents")
        .to_request();

    let unauth_resp = test::call_service(&app, unauth_req).await;
    let status = unauth_resp.status();
    println!("Unauthorized request: Status = {}", status);
    if status.as_u16() != 401 {
        let body = test::read_body(unauth_resp).await;
        println!("Unauthorized response body: {:?}", String::from_utf8_lossy(&body));
    }
    assert_eq!(status.as_u16(), 401);
}
