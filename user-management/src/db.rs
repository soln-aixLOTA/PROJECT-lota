use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use tokio_postgres::NoTls;
use sqlx::postgres::{PgPool, PgPoolOptions};
use anyhow::Result;

use crate::config::Config;

pub async fn init_db(config: &Config) -> Result<PgPool> {
    // Create the database pool with the specified configuration
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    // Run database migrations
    run_migrations(&pool).await?;

    // Initialize default data if needed
    init_default_data(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Read migration files from the migrations directory
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    Ok(())
}

async fn init_default_data(pool: &PgPool) -> Result<()> {
    // Check if default permissions exist
    let permissions_exist = sqlx::query!(
        "SELECT COUNT(*) as count FROM permissions"
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0) > 0;

    if !permissions_exist {
        // Insert default permissions
        sqlx::query!(
            r#"
            INSERT INTO permissions (name, description, resource_type, action)
            VALUES
                ('user.create', 'Create new users', 'user', 'create'),
                ('user.read', 'View user details', 'user', 'read'),
                ('user.update', 'Update user details', 'user', 'update'),
                ('user.delete', 'Delete users', 'user', 'delete'),
                ('role.create', 'Create new roles', 'role', 'create'),
                ('role.read', 'View role details', 'role', 'read'),
                ('role.update', 'Update role details', 'role', 'update'),
                ('role.delete', 'Delete roles', 'role', 'delete'),
                ('bot.create', 'Create new bots', 'bot', 'create'),
                ('bot.read', 'View bot details', 'bot', 'read'),
                ('bot.update', 'Update bot details', 'bot', 'update'),
                ('bot.delete', 'Delete bots', 'bot', 'delete')
            "#
        )
        .execute(pool)
        .await?;
    }

    // Check if default roles exist
    let roles_exist = sqlx::query!(
        "SELECT COUNT(*) as count FROM roles WHERE is_system_role = true"
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0) > 0;

    if !roles_exist {
        // Insert default system roles
        sqlx::query!(
            r#"
            INSERT INTO roles (name, description, is_system_role)
            VALUES
                ('tenant_admin', 'Full access to all tenant resources', true),
                ('user_manager', 'Can manage users and roles', true),
                ('bot_manager', 'Can manage bots', true),
                ('bot_user', 'Can use bots', true)
            "#
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

// Helper function to get a database connection from the pool
pub async fn get_connection(pool: &PgPool) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>> {
    Ok(pool.acquire().await?)
}

// Transaction helper
pub async fn transaction<F, R>(pool: &PgPool, f: F) -> Result<R>
where
    F: for<'c> FnOnce(&'c mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<R> + Send + 'static,
    R: Send + 'static,
{
    let mut tx = pool.begin().await?;
    let result = f(&mut tx).await?;
    tx.commit().await?;
    Ok(result)
} 