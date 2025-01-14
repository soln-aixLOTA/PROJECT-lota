use crate::db::users::{self, User as DbUser};
use crate::error::AppError;
use crate::models::auth::{CreateUserRequest, LoginRequest, LoginResponse};
use crate::models::user::User;
use crate::auth::jwt::JwtAuth;
use actix_web::{post, get, web, HttpResponse, HttpRequest};
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;
use validator::Validate;
use std::env;
use uuid::Uuid;

pub type AppResult<T> = Result<T, AppError>;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(refresh_token)
            .service(validate),
    );
}

#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    req: web::Json<CreateUserRequest>,
) -> AppResult<HttpResponse> {
    println!("Starting registration for user: {}", req.username);

    if let Err(e) = req.validate() {
        println!("Validation error: {:?}", e);
        return Err(AppError::Validation(e.to_string()));
    }

    let password_hash = match hash(req.password.as_bytes(), DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            println!("Password hashing error: {}", e);
            return Err(AppError::Internal(format!("Failed to hash password: {}", e)));
        }
    };

    println!("Creating user in database...");
    let db_user = match users::create_user(
        &**pool,
        req.username.clone(),
        req.email.clone(),
        password_hash,
        req.role.clone(),
    ).await {
        Ok(user) => user,
        Err(e) => {
            println!("Database error: {:?}", e);
            return Err(e);
        }
    };

    let user = User::from(db_user);
    println!("User created successfully with ID: {}", user.id);

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let roles = vec![user.role.to_string()];

    let access_token = jwt_auth.create_access_token(&user.id.to_string(), roles.clone())
        .map_err(|e| {
            println!("Access token generation error: {:?}", e);
            e
        })?;

    let refresh_token_str = jwt_auth.create_refresh_token(&user.id.to_string(), roles)
        .map_err(|e| {
            println!("Refresh token generation error: {:?}", e);
            e
        })?;

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
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role as "role!: String", created_at, updated_at
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
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let roles = vec![user.role.to_string()];

    let access_token = jwt_auth.create_access_token(&user.id.to_string(), roles.clone())?;
    let refresh_token_str = jwt_auth.create_refresh_token(&user.id.to_string(), roles)?;

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
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());

    // Validate the refresh token
    let claims = jwt_auth.validate_token(&token)?;

    // Fetch the user to ensure they still exist and get their current roles
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".into()))?;

    let db_user = sqlx::query_as!(
        DbUser,
        r#"
        SELECT id, username, email, password_hash, mfa_enabled, mfa_secret, role as "role!: String", created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&**pool)
    .await
    .map_err(|_| AppError::Authentication("User not found".into()))?;

    let user = User::from(db_user);
    let roles = vec![user.role.to_string()];

    // Generate new tokens
    let new_access_token = jwt_auth.create_access_token(&user.id.to_string(), roles.clone())?;
    let new_refresh_token_str = jwt_auth.create_refresh_token(&user.id.to_string(), roles)?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token_str,
        user_id: user.id,
    }))
}

#[get("/validate")]
pub async fn validate(req: HttpRequest) -> AppResult<HttpResponse> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Authentication("Missing Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Authentication("Invalid Authorization header format".into()))?;

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_auth = JwtAuth::new(jwt_secret.as_bytes());
    let claims = jwt_auth.validate_token(token)
        .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": claims.sub,
        "roles": claims.roles
    })))
}
