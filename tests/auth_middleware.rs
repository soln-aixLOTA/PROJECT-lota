use actix_web::{http::header, test, web, App};
use jsonwebtoken::{encode, EncodingKey, Header};
use lotabots::middleware::auth::{validate_token, Claims};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[actix_rt::test]
async fn test_validate_token_valid() {
    // Generate a test JWT
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        sub: "testuser".to_string(),
        exp: (now + 3600) as usize, // Expires in 1 hour
    };
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    // Validate the token
    let result = validate_token(&token);

    // Assert the token is valid
    assert!(result.is_ok());
    assert_eq!(result.unwrap().sub, "testuser");
}

#[actix_rt::test]
async fn test_validate_token_invalid() {
    // Generate an invalid token (e.g., expired)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        sub: "testuser".to_string(),
        exp: (now - 3600) as usize, // Expired 1 hour ago
    };
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    // Validate the token
    let result = validate_token(&token);

    // Assert the token is invalid
    assert!(result.is_err());
}

#[actix_rt::test]
async fn test_auth_middleware_valid_token() {
    // Generate a test JWT
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        sub: "testuser".to_string(),
        exp: (now + 3600) as usize, // Expires in 1 hour
    };
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    let mut app = test::init_service(
        App::new()
            .wrap(super::AuthMiddleware) // Assuming AuthMiddleware is in the same module
            .route("/", web::get().to(|| async { "Authorized" })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    assert_eq!(resp.status(), 200);
}
