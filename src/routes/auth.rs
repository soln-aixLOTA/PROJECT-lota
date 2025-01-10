use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::services::auth_service::AuthService;
use crate::services::user_service::UserService;
use crate::error::ServiceError;

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[post("/login")]
pub async fn login(
    credentials: web::Json<LoginCredentials>,
    user_service: web::Data<UserService>,
) -> Result<HttpResponse, ServiceError> {
    let user = user_service
        .authenticate_user(&credentials.username, &credentials.password)
        .await?;

    let token = AuthService::generate_token(user.id.to_string())?;
    Ok(HttpResponse::Ok().json(LoginResponse { token }))
}
