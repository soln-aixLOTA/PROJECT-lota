use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    db::Database,
    error::{Result, UserError},
    models::User,
};

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh_token)),
    );
}

async fn login(
    db: web::Data<Database>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    request.validate().map_err(|e| UserError::Validation(e.to_string()))?;

    // Find user by email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        FROM users
        WHERE email = $1 AND is_active = true
        "#,
        request.email,
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::Authentication("Invalid credentials".to_string()))?;

    // Verify password
    let is_valid = argon2::verify_encoded(&user.password_hash, request.password.as_bytes())
        .map_err(|e| UserError::Internal(e.to_string()))?;

    if !is_valid {
        return Err(UserError::Authentication("Invalid credentials".to_string()));
    }

    // Generate tokens
    let access_token = generate_access_token(&user)?;
    let refresh_token = generate_refresh_token(&user)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    }))
}

async fn refresh_token(
    db: web::Data<Database>,
    request: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse> {
    // Verify refresh token and get user ID
    let user_id = verify_refresh_token(&request.refresh_token)?;

    // Get user details
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
        user_id,
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::Authentication("Invalid refresh token".to_string()))?;

    // Generate new tokens
    let access_token = generate_access_token(&user)?;
    let refresh_token = generate_refresh_token(&user)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    }))
}

fn generate_access_token(user: &User) -> Result<String> {
    // Implementation using JWT from common library
    // This is a placeholder - actual implementation would use the common auth module
    Ok("access_token".to_string())
}

fn generate_refresh_token(user: &User) -> Result<String> {
    // Implementation using JWT from common library
    // This is a placeholder - actual implementation would use the common auth module
    Ok("refresh_token".to_string())
}

fn verify_refresh_token(token: &str) -> Result<uuid::Uuid> {
    // Implementation using JWT from common library
    // This is a placeholder - actual implementation would use the common auth module
    Ok(uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_login() {
        // Add test implementation
    }

    #[actix_rt::test]
    async fn test_refresh_token() {
        // Add test implementation
    }
} 