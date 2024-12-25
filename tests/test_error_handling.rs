#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_authentication_error() {
        let app = test::init_service(App::new().service(your_service_here)).await;
        let req = test::TestRequest::post()
            .uri("/your_authentication_endpoint")
            .set_json(json!({"username": "wrong_user", "password": "wrong_pass"}))
            .to_request();

        let resp: HttpResponse = app.call(req).await.unwrap();
        assert_eq!(resp.status(), HttpStatus::UNAUTHORIZED);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Authentication error");
    }

    #[actix_web::test]
    async fn test_validation_error() {
        let app = test::init_service(App::new().service(your_service_here)).await;
        let req = test::TestRequest::post()
            .uri("/your_validation_endpoint")
            .set_json(json!({"invalid_field": "value"}))
            .to_request();

        let resp: HttpResponse = app.call(req).await.unwrap();
        assert_eq!(resp.status(), HttpStatus::BAD_REQUEST);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Validation error");
    }
}
