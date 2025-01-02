#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn test_get_user() {
        let app = test::init_service(
            App::new().configure(handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/users?id=123")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
} 