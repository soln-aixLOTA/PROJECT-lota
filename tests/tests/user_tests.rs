use actix_web::{test, web, App};
use chrono::Utc;
use uuid::Uuid;

use crate::common::setup_test_db;
use crate::models::user::{CreateUser, User};
use crate::routes::{create_user, get_user, get_users};

#[actix_web::test]
async fn test_create_user() {
    let pool = setup_test_db().await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(create_user),
    )
    .await;

    let new_user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&new_user)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let user: User = test::read_body_json(resp).await;
    assert_eq!(user.username, new_user.username);
    assert_eq!(user.email, new_user.email);
}
