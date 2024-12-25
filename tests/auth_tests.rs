use actix_web::{test, web, App};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::env;

use crate::common::setup_test_db;
use crate::test_handler;

use super::super::{
    middleware::auth::{Claims, JwtAuth},
    routes::auth::{login, LoginCredentials, LoginResponse},
};

#[actix_web::test]
async fn test_jwt_auth_valid_token() {
    let token = create_test_token(3600, vec!["user".to_string()]);

    let app = test::init_service(
        App::new()
            .wrap(JwtAuth::new())
            .route("/", web::get().to(test_handler)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_jwt_auth_expired_token() {
    let token = create_test_token(-3600, vec!["user".to_string()]);

    let app = test::init_service(
        App::new()
            .wrap(JwtAuth::new())
            .route("/", web::get().to(test_handler)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_jwt_auth_missing_token() {
    let app = test::init_service(
        App::new()
            .wrap(JwtAuth::new())
            .route("/", web::get().to(test_handler)),
    )
    .await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_login_success() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(login),
    )
    .await;

    let credentials = LoginCredentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&credentials)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: LoginResponse = test::read_body_json(resp).await;
    assert!(!body.token.is_empty());
    assert_eq!(body.token_type, "Bearer");
    assert_eq!(body.expires_in, 3600);
}

#[actix_web::test]
async fn test_login_invalid_credentials() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(login),
    )
    .await;

    let credentials = LoginCredentials {
        username: "nonexistent".to_string(),
        password: "wrongpassword".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&credentials)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

fn create_test_token(exp_offset: i64, roles: Vec<String>) -> String {
    let exp = Utc::now().timestamp() as usize + exp_offset as usize;
    let claims = Claims {
        sub: "test_user".to_string(),
        exp,
        iat: Utc::now().timestamp() as usize,
        tenant_id: "test_tenant".to_string(),
        roles,
        permissions: vec![],
    };

    let secret = "test_secret";
    env::set_var("JWT_SECRET", secret);

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
