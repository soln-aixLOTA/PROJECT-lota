use actix_web::{
    test,
    web::{self, ServiceConfig},
    App,
};
use api_gateway::{
    middleware::auth::AuthMiddleware,
    routes::{health, proxy},
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn create_test_token(secret: &str, expiry_secs: u64) -> String {
    let exp = if expiry_secs == 0 {
        // For expired tokens, set expiration to 1 hour ago
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(3600)
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + expiry_secs
    };

    let claims = Claims {
        sub: "test_user".to_string(),
        exp: exp as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

fn app_config(cfg: &mut ServiceConfig) {
    cfg.app_data(web::Data::new(proxy::ServiceRegistry::new()))
        .service(health::health_check)
        .service(web::scope("/api/v1").service(proxy::proxy_route));
}

#[actix_rt::test]
async fn test_health_check_no_auth() {
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_secret".to_string()))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_protected_route_no_auth() {
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_secret".to_string()))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/test/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 401);
}

#[actix_rt::test]
async fn test_protected_route_valid_token() {
    let secret = "test_secret";
    let token = create_test_token(secret, 3600);

    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new(secret.to_string()))
            .configure(app_config),
    )
    .await;

    // Test GET request
    let req = test::TestRequest::get()
        .uri("/api/v1/test/test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 200);

    // Test POST request
    let req = test::TestRequest::post()
        .uri("/api/v1/test/test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 200);
}

#[actix_rt::test]
async fn test_protected_route_expired_token() {
    let secret = "test_secret";
    let token = create_test_token(secret, 0); // Expired token

    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new(secret.to_string()))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/test/test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 401);
}

#[actix_rt::test]
async fn test_protected_route_invalid_token() {
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_secret".to_string()))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/test/test")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 401);
}

#[actix_rt::test]
async fn test_protected_route_malformed_header() {
    let app = test::init_service(
        App::new()
            .wrap(AuthMiddleware::new("test_secret".to_string()))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/test/test")
        .insert_header(("Authorization", "not_a_bearer_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert_eq!(status, 401);
}
