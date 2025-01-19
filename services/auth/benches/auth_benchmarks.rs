use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use lotabots_auth::{
    models::User,
    repository::AuthRepository,
    service::AuthService,
    LoginRequest, RegisterRequest,
};

async fn setup_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/lotabots_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn bench_password_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Password Operations");
    group.sample_size(50); // Reduced sample size due to expensive operation

    group.bench_function("hash_password", |b| {
        b.iter(|| {
            bcrypt::hash(
                black_box("test_password_123!@#"),
                black_box(bcrypt::DEFAULT_COST),
            )
        })
    });

    let hash = bcrypt::hash("test_password_123!@#", bcrypt::DEFAULT_COST).unwrap();
    group.bench_function("verify_password", |b| {
        b.iter(|| {
            bcrypt::verify(
                black_box("test_password_123!@#"),
                black_box(&hash),
            )
        })
    });

    group.finish();
}

fn bench_jwt_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("JWT Operations");

    let user_id = Uuid::new_v4();
    let secret = "TEST_SECRET_BENCHMARK_ONLY";
    let service = AuthService::new(
        AuthRepository::new(
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(setup_test_db())
        ),
        secret.to_string(),
    );

    group.bench_function("generate_token", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                black_box(service.generate_token(black_box(user_id)))
            })
    });

    group.finish();
}

fn bench_user_registration(c: &mut Criterion) {
    let mut group = c.benchmark_group("User Operations");
    group.sample_size(30); // Reduced sample size due to database operations

    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(setup_test_db());
    let service = AuthService::new(
        AuthRepository::new(pool),
        "test_secret".to_string(),
    );

    group.bench_function("register_user", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let req = RegisterRequest {
                    username: format!("user_{}", Uuid::new_v4()),
                    email: format!("user_{}@example.com", Uuid::new_v4()),
                    password: "Password123!@#".to_string(),
                };
                black_box(service.register(black_box(req))).await
            })
    });

    group.finish();
}

fn bench_user_login(c: &mut Criterion) {
    let mut group = c.benchmark_group("Login Operations");
    group.sample_size(30); // Reduced sample size due to database operations

    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(setup_test_db());
    let service = AuthService::new(
        AuthRepository::new(pool.clone()),
        "test_secret".to_string(),
    );

    // Create a test user first
    let username = "benchmark_user";
    let password = "Password123!@#";
    rt.block_on(async {
        let req = RegisterRequest {
            username: username.to_string(),
            email: "benchmark@example.com".to_string(),
            password: password.to_string(),
        };
        service.register(req).await.unwrap();
    });

    group.bench_function("login_user", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap())
            .iter(|| async {
                let req = LoginRequest {
                    username: username.to_string(),
                    password: password.to_string(),
                };
                black_box(service.login(black_box(req))).await
            })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_password_hashing,
    bench_jwt_operations,
    bench_user_registration,
    bench_user_login
);
criterion_main!(benches);
