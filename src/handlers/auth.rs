use crate::{
    auth::{create_access_token, create_refresh_token, validate_token, Claims},
    error::AppError,
    models::user::{CreateUserRequest, User},
};
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[post("/register")]
pub async fn register(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user = User::create(&pool, req.0).await?;
    let claims = Claims::new(user.id.to_string(), user.role.clone());

    let access_token = create_access_token(claims.clone())?;
    let refresh_jwt = create_refresh_token(claims)?;

    Ok(HttpResponse::Created().json(json!({
        "user": user,
        "access_token": access_token,
        "refresh_token": refresh_jwt,
    })))
}

#[post("/login")]
pub async fn login(
    pool: web::Data<sqlx::PgPool>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let user = User::authenticate(&pool, &req.email, &req.password).await?;
    let claims = Claims::new(user.id.to_string(), user.role.clone());

    let access_token = create_access_token(claims.clone())?;
    let refresh_jwt = create_refresh_token(claims)?;

    Ok(HttpResponse::Ok().json(json!({
        "user": user,
        "access_token": access_token,
        "refresh_token": refresh_jwt,
    })))
}

#[post("/refresh")]
pub async fn refresh_token(req: web::Json<RefreshTokenRequest>) -> Result<HttpResponse, AppError> {
    let claims = validate_token(&req.refresh_token)?;
    let new_access_token = create_access_token(claims.clone())?;
    let new_refresh_jwt = create_refresh_token(claims)?;

    Ok(HttpResponse::Ok().json(json!({
        "access_token": new_access_token,
        "refresh_token": new_refresh_jwt,
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh_token),
    );
}
