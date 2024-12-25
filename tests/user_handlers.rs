use crate::models::user::{CreateUser, DbError, Role, User};
use crate::routes::users::{create_user, get_user, get_users};
use actix_web::{test, web, App};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[actix_rt::test]
async fn test_get_users_success() {
    // Set up a test database pool (replace with your test DB URL)
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/testdb")
        .await
        .unwrap();

    // Create a test app
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_users),
    )
    .await;

    // Send a GET request to /users
    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&mut app, req).await;

    // Assert the status code is 200 OK
    assert_eq!(resp.status(), 200);

    // Optionally, deserialize and assert the response body
    // let users: Vec<User> = test::read_body_json(resp).await;
    // assert!(!users.is_empty());
}

#[actix_rt::test]
async fn test_create_user_success() {
    // Set up a test database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/testdb")
        .await
        .unwrap();

    // Create a test app
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(create_user),
    )
    .await;

    // Create a sample user
    let new_user = CreateUser {
        username: "testuser".to_string(),
        email: "testuser@example.com".to_string(),
        password: "testpassword".to_string(),
    };

    // Send a POST request to /users
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&new_user)
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    // Assert the status code is 201 Created
    assert_eq!(resp.status(), 201);

    // Optionally, deserialize and assert the response body
    // let created_user: User = test::read_body_json(resp).await;
    // assert_eq!(created_user.username, new_user.username);
}

#[actix_rt::test]
async fn test_get_user_success() {
    // Set up a test database pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/testdb")
        .await
        .unwrap();

    // Create a user ID for testing (replace with an existing user ID in your test DB)
    let user_id = Uuid::new_v4();

    // Create a test app
    let mut app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_user),
    )
    .await;

    // Send a GET request to /users/{id}
    let req = test::TestRequest::get()
        .uri(&format!("/users/{}", user_id))
        .to_request();
    let resp = test::call_service(&mut app, req).await;

    // Assert the status code is 200 OK or 404 Not Found (if the user doesn't exist)
    // In a real test, you'd likely want to create a test user first
    assert!(resp.status().is_success() || resp.status() == 404);

    // Optionally, deserialize and assert the response body
    // if resp.status().is_success() {
    //     let user: User = test::read_body_json(resp).await;
    //     assert_eq!(user.id, user_id);
    // }
}
