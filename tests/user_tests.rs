use actix_web::{test, web, App};
use lotabots::handlers::user_handler as user_handlers;

#[actix_web::test]
async fn test_get_users() {
    let app =
        test::init_service(App::new().route("/users", web::get().to(user_handlers::get_users)))
            .await;
    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
