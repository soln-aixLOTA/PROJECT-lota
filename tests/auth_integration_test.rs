#[cfg(test)]
mod tests {
    use crate::auth::jwt::{create_jwt, validate_jwt};
    use crate::errors::ApiError;
    use crate::handlers::auth::{
        login, register, validate_token, AuthRequest, ValidateTokenRequest,
    };
    use crate::models::user::{CreateUserRequest, UserRole};
    use crate::repositories::user::UserRepository;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use dotenv::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool")
    }

    #[actix_web::test]
    async fn test_register_login_validate_token() {
        let pool = setup_db().await;
        let user_repo = web::Data::new(UserRepository::new(pool.clone()));

        // Test registration
        let register_req = web::Json(AuthRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        });
        let register_resp = test::call_service(
            &mut test::init_service(
                App::new()
                    .app_data(user_repo.clone())
                    .route("/register", web::post().to(register)),
            )
            .await,
            test::TestRequest::post()
                .uri("/register")
                .set_json(&register_req)
                .to_request(),
        )
        .await;
        assert_eq!(register_resp.status(), StatusCode::CREATED);
        let register_body: crate::handlers::auth::AuthResponse =
            test::read_body_json(register_resp).await;
        let token = register_body.token;

        // Test login
        let login_req = web::Json(AuthRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        });
        let login_resp = test::call_service(
            &mut test::init_service(
                App::new()
                    .app_data(user_repo.clone())
                    .route("/login", web::post().to(login)),
            )
            .await,
            test::TestRequest::post()
                .uri("/login")
                .set_json(&login_req)
                .to_request(),
        )
        .await;
        assert_eq!(login_resp.status(), StatusCode::OK);
        let login_body: crate::handlers::auth::AuthResponse =
            test::read_body_json(login_resp).await;
        assert_eq!(login_body.token, token);

        // Test token validation
        let validate_req = web::Json(ValidateTokenRequest {
            token: token.clone(),
        });
        let validate_resp = test::call_service(
            &mut test::init_service(App::new().route("/validate", web::post().to(validate_token)))
                .await,
            test::TestRequest::post()
                .uri("/validate")
                .set_json(&validate_req)
                .to_request(),
        )
        .await;
        assert_eq!(validate_resp.status(), StatusCode::OK);
        let validate_body: crate::handlers::auth::ValidateTokenResponse =
            test::read_body_json(validate_resp).await;
        let user_id = validate_jwt(&token).unwrap();
        assert_eq!(validate_body.user_id, user_id);

        // Test invalid login
        let invalid_login_req = web::Json(AuthRequest {
            username: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        });
        let invalid_login_resp = test::call_service(
            &mut test::init_service(
                App::new()
                    .app_data(user_repo.clone())
                    .route("/login", web::post().to(login)),
            )
            .await,
            test::TestRequest::post()
                .uri("/login")
                .set_json(&invalid_login_req)
                .to_request(),
        )
        .await;
        assert_eq!(invalid_login_resp.status(), StatusCode::UNAUTHORIZED);

        // Test invalid token
        let invalid_validate_req = web::Json(ValidateTokenRequest {
            token: "invalidtoken".to_string(),
        });
        let invalid_validate_resp = test::call_service(
            &mut test::init_service(App::new().route("/validate", web::post().to(validate_token)))
                .await,
            test::TestRequest::post()
                .uri("/validate")
                .set_json(&invalid_validate_req)
                .to_request(),
        )
        .await;
        assert_eq!(
            invalid_validate_resp.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
