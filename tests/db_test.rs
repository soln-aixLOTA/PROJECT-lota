use dotenvy::dotenv;
use sqlx::postgres::PgPool;
use std::env;

#[actix_web::test]
async fn test_db_operations() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await.unwrap();

    // Test user creation
    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash, email) VALUES ($1, $2, $3) RETURNING id",
        "testuser",
        "hashedpassword",
        "test@example.com"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(result.id.to_string().len() > 0);

    // Test user retrieval
    let user = sqlx::query!(
        "SELECT username, email FROM users WHERE id = $1",
        result.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");

    // Cleanup
    sqlx::query!("DELETE FROM users WHERE id = $1", result.id)
        .execute(&pool)
        .await
        .unwrap();
}
