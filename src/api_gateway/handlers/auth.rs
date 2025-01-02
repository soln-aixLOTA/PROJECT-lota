use crate::auth::jwt::{create_jwt, create_jwt_with_refresh, refresh_jwt, validate_jwt};
use crate::errors::ApiError;
use crate::repositories::user::UserRepository;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8, max = 100), custom = "validate_password")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub roles: Vec<String>,
}

fn validate_password(password: &str) -> Result<(), validator::ValidationError> {
    // Password must contain at least one uppercase letter, one lowercase letter,
    // one number, and one special character
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_number = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase || !has_lowercase || !has_number || !has_special {
        return Err(validator::ValidationError::new(
            "Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character",
        ));
    }

    Ok(())
}

#[instrument(skip(user_repo))]
pub async fn login(
    auth_req: web::Json<AuthRequest>,
    user_repo: web::Data<UserRepository>,
) -> Result<impl Responder, ApiError> {
    // Validate request
    auth_req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::ValidationError(e.to_string())
    })?;

    let user = user_repo
        .find_by_username(&auth_req.username)
        .await
        .map_err(|e| {
            error!("Database error while finding user: {}", e);
            ApiError::from(e)
        })?
        .ok_or(ApiError::UserNotFound)?;

    if user
        .verify_password(&auth_req.password)
        .map_err(|_| ApiError::PasswordMismatch)?
    {
        let (token, refresh_token, expires_in) =
            create_jwt_with_refresh(user.id.to_string(), Some(user.roles.clone())).map_err(
                |e| {
                    error!("JWT error while creating token: {}", e);
                    ApiError::from(e)
                },
            )?;

        info!("User {} logged in successfully", user.username);
        Ok(HttpResponse::Ok().json(AuthResponse {
            token,
            refresh_token,
            expires_in,
            roles: user.roles,
        }))
    } else {
        error!("Invalid credentials for user: {}", auth_req.username);
        Err(ApiError::InvalidCredentials)
    }
}

#[instrument(skip(user_repo))]
pub async fn register(
    auth_req: web::Json<AuthRequest>,
    user_repo: web::Data<UserRepository>,
) -> Result<impl Responder, ApiError> {
    // Validate request
    auth_req.validate().map_err(|e| {
        error!("Validation error: {}", e);
        ApiError::ValidationError(e.to_string())
    })?;

    let new_user = user_repo
        .create(&auth_req.username, &auth_req.password)
        .await
        .map_err(|e| {
            error!("Database error while creating user: {}", e);
            ApiError::from(e)
        })?;

    let (token, refresh_token, expires_in) =
        create_jwt_with_refresh(new_user.id.to_string(), Some(new_user.roles.clone())).map_err(
            |e| {
                error!("JWT error while creating token: {}", e);
                ApiError::from(e)
            },
        )?;

    info!("User {} registered successfully", new_user.username);
    Ok(HttpResponse::Created().json(AuthResponse {
        token,
        refresh_token,
        expires_in,
        roles: new_user.roles,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTokenResponse {
    pub user_id: String,
    pub roles: Vec<String>,
}

#[instrument]
pub async fn validate_token(
    token_req: web::Json<ValidateTokenRequest>,
) -> Result<impl Responder, ApiError> {
    let (user_id, roles) = validate_jwt(&token_req.token).map_err(|e| {
        error!("JWT error while validating token: {}", e);
        ApiError::from(e)
    })?;

    Ok(HttpResponse::Ok().json(ValidateTokenResponse {
        user_id,
        roles: roles.unwrap_or_default(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[instrument]
pub async fn refresh_token(
    refresh_req: web::Json<RefreshTokenRequest>,
) -> Result<impl Responder, ApiError> {
    let (token, refresh_token, expires_in) =
        refresh_jwt(&refresh_req.refresh_token).map_err(|e| {
            error!("JWT error while refreshing token: {}", e);
            ApiError::from(e)
        })?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        refresh_token,
        expires_in,
        roles: vec![], // Roles are included in the token
    }))
}
