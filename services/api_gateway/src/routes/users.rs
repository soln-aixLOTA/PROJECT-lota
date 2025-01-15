use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, DbPool},
    error::ApiError,
    middleware::Claims,
    models::{ApiResponse, CreateUserRequest, LoginResponse, User, UserLogin},
    utils::validate_password,
};

pub fn configure_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/login")
            .route(web::post().to(login)),
    )
    .service(
        web::resource("/register")
            .route(web::post().to(create_user)),
    );
}

pub fn configure_users(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(list_users))
        .route("/{id}", web::get().to(get_user))
        .route("/{id}", web::put().to(update_user))
        .route("/{id}", web::delete().to(delete_user));
}

pub async fn create_user(
    pool: web::Data<DbPool>,
    user_data: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    user_data.validate()?;

    // Validate password strength
    validate_password(&user_data.password, &[&user_data.username, &user_data.email])?;

    // Hash password
    let hashed_password = hash(user_data.password.as_bytes(), DEFAULT_COST)?;

    // Create user in database
    let user_id = db::create_user(
        &pool,
        &user_data.username,
        &user_data.email,
        &hashed_password,
    )
    .await?;

    // Fetch created user
    let user = db::get_user_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch created user".to_string()))?;

    Ok(HttpResponse::Created().json(ApiResponse::success(user)))
}

pub async fn list_users(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let users = db::list_users(&pool, 100, 0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(users)))
}

pub async fn get_user(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user = db::get_user_by_id(&pool, id.into_inner())
        .await?
        .ok_or_else(|| ApiError::NotFoundError("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(user)))
}

pub async fn update_user(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    user_data: web::Json<User>,
) -> Result<HttpResponse, ApiError> {
    let user_id = id.into_inner();

    // Get existing user to validate against
    let existing_user = db::get_user_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFoundError("User not found".to_string()))?;

    // If password is being updated, validate it
    if let Some(ref password) = user_data.password {
        validate_password(password, &[&existing_user.username, &existing_user.email])?;

        // Hash new password
        let hashed_password = hash(password.as_bytes(), DEFAULT_COST)?;

        // Update user with new password
        db::update_user_with_password(
            &pool,
            user_id,
            &user_data.username,
            &user_data.email,
            &hashed_password,
        )
        .await?;
    } else {
        // Update user without changing password
        db::update_user(
            &pool,
            user_id,
            &user_data.username,
            &user_data.email,
        )
        .await?;
    }

    // Fetch updated user
    let updated_user = db::get_user_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch updated user".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated_user)))
}

pub async fn delete_user(
    _pool: web::Data<DbPool>,
    _id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    // TODO: Implement user deletion logic
    Err(ApiError::InternalError("Not implemented".to_string()))
}

pub async fn login(
    pool: web::Data<DbPool>,
    credentials: web::Json<UserLogin>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    credentials.validate()?;

    // Basic length check before database query
    if credentials.password.len() < 8 {
        return Err(ApiError::AuthenticationError("Invalid credentials".to_string()));
    }

    // Get user from database
    let (user_id, _, password_hash) = db::get_user_by_email(&pool, &credentials.email)
        .await?
        .ok_or_else(|| ApiError::AuthenticationError("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(credentials.password.as_bytes(), &password_hash)? {
        return Err(ApiError::AuthenticationError("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
    };

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    // Get user data
    let user = db::get_user_by_id(&pool, user_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch user data".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(LoginResponse { token, user })))
}
