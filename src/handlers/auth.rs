use crate::db::users::User as DbUser;
use crate::error::AppError;
use crate::models::auth::{CreateUserRequest, LoginRequest, LoginResponse};
use crate::models::user::User;
use crate::auth::{generate_access_token, generate_refresh_token, validate_refresh_token};
use actix_web::{post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub type AppResult<T> = Result<T, AppError>;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh_token),
    );
}

#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<CreateUserRequest>,
) -> AppResult<HttpResponse> {
    req.validate()?;

    let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    let db_user = sqlx::query_as!(
        DbUser,
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        "#,
        req.username,
        req.email,
        password_hash,
        req.role.to_string().to_lowercase(),
    )
    .fetch_one(&**pool)
    .await?;

    let user = User::from(db_user);
    let access_token = generate_access_token(&user)?;
    let refresh_token_str = generate_refresh_token(&user)?;

    Ok(HttpResponse::Created().json(LoginResponse {
        access_token,
        refresh_token: refresh_token_str,
        user_id: user.id,
    }))
}

#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    req: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    req.validate()?;

    let db_user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        FROM users
        WHERE username = $1
        "#,
        req.username
    )
    .fetch_one(&**pool)
    .await
    .map_err(|_| AppError::Authentication("Invalid username or password".into()))?;

    if !verify(&req.password, &db_user.password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))? {
        return Err(AppError::Authentication("Invalid username or password".into()));
    }

    let user = User::from(db_user);
    let access_token = generate_access_token(&user)?;
    let refresh_token_str = generate_refresh_token(&user)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token: refresh_token_str,
        user_id: user.id,
    }))
}

#[post("/refresh")]
pub async fn refresh_token(
    pool: web::Data<PgPool>,
    token: web::Json<String>,
) -> AppResult<HttpResponse> {
    let user_id = validate_refresh_token(&token)?;

    let db_user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&**pool)
    .await
    .map_err(|_| AppError::Authentication("Invalid refresh token".into()))?;

    let user = User::from(db_user);
    let access_token = generate_access_token(&user)?;
    let refresh_token_str = generate_refresh_token(&user)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token: refresh_token_str,
        user_id: user.id,
    }))
}
