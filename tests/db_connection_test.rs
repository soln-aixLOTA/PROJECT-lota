use dotenv::dotenv;
use sqlx::postgres::PgPool;
use std::env;

#[actix_web::test]
async fn test_db_connection() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();

    // Test a simple query
    let result = sqlx::query!("SELECT 1 as one")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(result.one, Some(1));
}

#[actix_web::test]
async fn test_db_migrations() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();

    // Test that migrations can be applied
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Test that users table exists
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'users')"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(result.exists.unwrap());
}
