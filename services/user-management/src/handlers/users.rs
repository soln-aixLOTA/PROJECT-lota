use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::Database,
    error::{Result, UserError},
    models::{CreateUserRequest, UpdateUserRequest, User},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(create_user))
            .route("", web::get().to(list_users))
            .route("/{id}", web::get().to(get_user))
            .route("/{id}", web::put().to(update_user))
            .route("/{id}", web::delete().to(delete_user)),
    );
}

async fn create_user(
    db: web::Data<Database>,
    request: web::Json<CreateUserRequest>,
) -> Result<HttpResponse> {
    request.validate().map_err(|e| UserError::Validation(e.to_string()))?;

    // Hash password and create user
    let password_hash = argon2::hash_encoded(
        request.password.as_bytes(),
        b"salt", // In production, use a proper salt
        &argon2::Config::default(),
    )
    .map_err(|e| UserError::Internal(e.to_string()))?;

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, name, tenant_id, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        "#,
        request.email,
        request.name,
        request.tenant_id,
        password_hash,
    )
    .fetch_one(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Created().json(user))
}

async fn list_users(
    db: web::Data<Database>,
    tenant_id: web::Query<Uuid>,
) -> Result<HttpResponse> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        FROM users
        WHERE tenant_id = $1 AND is_active = true
        "#,
        tenant_id.into_inner(),
    )
    .fetch_all(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Ok().json(users))
}

async fn get_user(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
        id.into_inner(),
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(user))
}

async fn update_user(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
    request: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse> {
    request.validate().map_err(|e| UserError::Validation(e.to_string()))?;

    let mut user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
        id.into_inner(),
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::NotFound("User not found".to_string()))?;

    if let Some(email) = &request.email {
        user.email = email.clone();
    }
    if let Some(name) = &request.name {
        user.name = name.clone();
    }
    if let Some(is_active) = request.is_active {
        user.is_active = is_active;
    }

    let updated_user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET email = $1, name = $2, is_active = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING id, email, name, tenant_id, is_active, created_at, updated_at, password_hash
        "#,
        user.email,
        user.name,
        user.is_active,
        user.id,
    )
    .fetch_one(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Ok().json(updated_user))
}

async fn delete_user(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    sqlx::query!(
        r#"
        UPDATE users
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        "#,
        id.into_inner(),
    )
    .execute(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_create_user() {
        // Add test implementation
    }

    #[actix_rt::test]
    async fn test_get_user() {
        // Add test implementation
    }
} 