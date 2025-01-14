use sqlx::postgres::PgPoolOptions;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("Error: DATABASE_URL environment variable not set");
        process::exit(1);
    });

    println!("Attempting to connect to database...");

    match PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
    {
        Ok(_) => {
            println!("✅ Successfully connected to the database!");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to the database: {}", e);
            process::exit(1);
        }
    }
}
