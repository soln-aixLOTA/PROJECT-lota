use fake::{Fake, Faker};
use sqlx::PgPool;
use testcontainers::{clients::Cli, Container, Docker};
use uuid::Uuid;

pub mod fixtures;
pub mod helpers;
pub mod mocks;

pub struct TestDatabase {
    pub pool: PgPool,
    _container: Container<'static, Cli, testcontainers::images::postgres::Postgres>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let docker = Cli::default();
        let container = docker.run(testcontainers::images::postgres::Postgres::default());
        let port = container.get_host_port_ipv4(5432);
        let connection_string = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            port
        );

        let pool = PgPool::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self {
            pool,
            _container: container,
        }
    }
}

pub fn generate_test_id() -> String {
    format!("test_{}", Uuid::new_v4())
}

pub fn random_string(len: usize) -> String {
    (0..len).map(|_| Faker.fake::<char>()).collect()
}

pub fn random_email() -> String {
    format!("{}@example.com", random_string(10))
}
