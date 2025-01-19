use actix_web::{test, web, App};
use document_automation::{
    handlers,
    models::auth::{CreateUserRequest, UserRole},
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidateTokenResponse {
    username: String,
}

async fn setup_db() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[actix_web::test]
async fn test_auth_flow() {
    let pool = setup_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(handlers::auth::config),
    )
    .await;

    // Test registration
    let register_request = CreateUserRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
        email: "test@example.com".to_string(),
        role: UserRole::User,
    };

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&register_request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let register_body: AuthResponse = test::read_body_json(resp).await;
    assert!(!register_body.token.is_empty());

    // Test login
    let login_request = serde_json::json!({
        "username": "testuser",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&login_request)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let login_body: AuthResponse = test::read_body_json(resp).await;
    assert!(!login_body.token.is_empty());

    // Test token validation
    let req = test::TestRequest::get()
        .uri("/auth/validate")
        .insert_header(("Authorization", format!("Bearer {}", login_body.token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let validate_body: ValidateTokenResponse = test::read_body_json(resp).await;
    assert_eq!(validate_body.username, "testuser");

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE username = $1", "testuser")
        .execute(&pool)
        .await
        .unwrap();
}
