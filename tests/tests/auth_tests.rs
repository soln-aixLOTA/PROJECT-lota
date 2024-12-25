use actix_web::http::header::ContentType;
use actix_web::{test, web, App};

use crate::middleware::auth::{Claims, JwtAuth};
use crate::routes::login;

async fn setup_test_app() -> actix_web::test::TestServer {
    // ... rest of your setup
}

#[actix_web::test]
async fn test_login() {
    // ... rest of your test
}
