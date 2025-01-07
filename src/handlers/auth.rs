use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::auth::{hash_password, verify_password, Claims};
use crate::core::AppState;
use crate::core::error::{AppError, AppResult};
use crate::db::users;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    let password_hash = hash_password(&req.password)?;
    let user = users::create_user(&state.db, req.username.clone(), password_hash).await?;

    let claims = Claims::new(user.id, user.username);
    let token = claims.to_token()?;

    Ok(HttpResponse::Created().json(AuthResponse { token }))
}

pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    let user = users::get_user_by_username(&state.db, &req.username)
        .await
        .map_err(|_| AppError::Auth("Invalid username or password".to_string()))?;

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Auth("Invalid username or password".to_string()));
    }

    let claims = Claims::new(user.id, user.username);
    let token = claims.to_token()?;

    Ok(HttpResponse::Ok().json(AuthResponse { token }))
} 