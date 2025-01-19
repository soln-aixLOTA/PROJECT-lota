use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use uuid::Uuid;
use crate::models::{User, Product};
use crate::error::ApiError;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}

// User queries
pub async fn create_user(
    pool: &DbPool,
    username: &str,
    email: &str,
    hashed_password: &str,
) -> Result<Uuid, ApiError> {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        "#,
        user_id,
        username,
        email,
        hashed_password,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to create user: {}", e)))?;

    Ok(user_id)
}

pub async fn get_user_by_id(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Option<User>, ApiError> {
    sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            username,
            email,
            NULL as password,
            created_at,
            updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to get user by id: {}", e)))
}

pub async fn get_user_by_email(
    pool: &DbPool,
    email: &str,
) -> Result<Option<(Uuid, String, String)>, ApiError> {
    sqlx::query!(
        r#"
        SELECT id, email, password_hash
        FROM users
        WHERE email = $1
        "#,
        email,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to get user by email: {}", e)))
    .map(|opt| opt.map(|row| (row.id, row.email, row.password_hash)))
}

pub async fn list_users(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, ApiError> {
    sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            username,
            email,
            NULL as password,
            created_at,
            updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to list users: {}", e)))
}

// Product queries
pub async fn create_product(
    pool: &DbPool,
    name: &str,
    description: Option<&str>,
    price: sqlx::types::BigDecimal,
    stock: i32,
) -> Result<Uuid, ApiError> {
    let product_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO products (id, name, description, price, stock, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        "#,
        product_id,
        name,
        description,
        price,
        stock,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to create product: {}", e)))?;

    Ok(product_id)
}

pub async fn get_product_by_id(
    pool: &DbPool,
    product_id: Uuid,
) -> Result<Option<Product>, ApiError> {
    sqlx::query_as!(
        Product,
        r#"
        SELECT
            id,
            name,
            description,
            price as "price: sqlx::types::BigDecimal",
            stock,
            metadata as "metadata: serde_json::Value",
            created_at,
            updated_at
        FROM products
        WHERE id = $1
        "#,
        product_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to get product by id: {}", e)))
}

pub async fn list_products(
    pool: &DbPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Product>, ApiError> {
    sqlx::query_as!(
        Product,
        r#"
        SELECT
            id,
            name,
            description,
            price as "price: sqlx::types::BigDecimal",
            stock,
            metadata as "metadata: serde_json::Value",
            created_at,
            updated_at
        FROM products
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to list products: {}", e)))
}

pub async fn update_product(
    pool: &DbPool,
    product_id: Uuid,
    name: &str,
    description: Option<&str>,
    price: sqlx::types::BigDecimal,
    stock: i32,
) -> Result<bool, ApiError> {
    let result = sqlx::query!(
        r#"
        UPDATE products
        SET name = $1, description = $2, price = $3, stock = $4, updated_at = NOW()
        WHERE id = $5
        "#,
        name,
        description,
        price,
        stock,
        product_id,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to update product: {}", e)))?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_product(
    pool: &DbPool,
    product_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM products
        WHERE id = $1
        "#,
        product_id,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to delete product: {}", e)))?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_user(
    pool: &DbPool,
    user_id: Uuid,
    username: &str,
    email: &str,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            username = $1,
            email = $2,
            updated_at = NOW()
        WHERE id = $3
        "#,
        username,
        email,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to update user: {}", e)))?;

    Ok(())
}

pub async fn update_user_with_password(
    pool: &DbPool,
    user_id: Uuid,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<(), ApiError> {
    sqlx::query!(
        r#"
        UPDATE users
        SET
            username = $1,
            email = $2,
            password_hash = $3,
            updated_at = NOW()
        WHERE id = $4
        "#,
        username,
        email,
        password_hash,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::InternalError(format!("Failed to update user: {}", e)))?;

    Ok(())
}
