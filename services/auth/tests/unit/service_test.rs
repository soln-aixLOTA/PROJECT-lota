use bcrypt::verify;
use mockall::predicate::*;
use mockall::mock;
use uuid::Uuid;

use lotabots_auth::{
    models::{User, UserResponse},
    repository::AuthRepository,
    service::AuthService,
    AuthError, LoginRequest, RegisterRequest,
};

// Mock the AuthRepository
mock! {
    Repository {}

    impl Clone for Repository {
        fn clone(&self) -> Self;
    }

    #[async_trait::async_trait]
    impl AuthRepository for Repository {
        fn new(_pool: sqlx::PgPool) -> Self;
        async fn create_user(&self, user: &User) -> lotabots_auth::Result<User>;
        async fn find_by_username(&self, username: &str) -> lotabots_auth::Result<Option<User>>;
        async fn find_by_email(&self, email: &str) -> lotabots_auth::Result<Option<User>>;
        async fn find_by_id(&self, id: Uuid) -> lotabots_auth::Result<Option<User>>;
    }
}

#[tokio::test]
async fn test_register_success() {
    let mut mock_repo = MockRepository::new();

    // Setup expectations
    mock_repo
        .expect_find_by_username()
        .with(eq("testuser"))
        .returning(|_| Ok(None));

    mock_repo
        .expect_find_by_email()
        .with(eq("test@example.com"))
        .returning(|_| Ok(None));

    mock_repo
        .expect_create_user()
        .returning(|user| Ok(user.clone()));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = RegisterRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = service.register(req).await;
    assert!(result.is_ok());

    let user_response = result.unwrap();
    assert_eq!(user_response.username, "testuser");
    assert_eq!(user_response.email, "test@example.com");
}

#[tokio::test]
async fn test_register_username_taken() {
    let mut mock_repo = MockRepository::new();

    // Setup expectations for existing user
    mock_repo
        .expect_find_by_username()
        .with(eq("testuser"))
        .returning(|_| Ok(Some(User::new(
            "testuser".to_string(),
            "existing@example.com".to_string(),
            "hash".to_string(),
        ))));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = RegisterRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = service.register(req).await;
    assert!(matches!(result, Err(AuthError::UsernameTaken)));
}

#[tokio::test]
async fn test_register_with_short_username() {
    let mut mock_repo = MockRepository::new();
    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = RegisterRequest {
        username: "ab".to_string(), // Too short (min is 3)
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = service.register(req).await;
    assert!(matches!(result, Err(AuthError::Validation(_))));
}

#[tokio::test]
async fn test_register_with_long_username() {
    let mut mock_repo = MockRepository::new();
    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = RegisterRequest {
        username: "a".repeat(51), // Too long (max is 50)
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = service.register(req).await;
    assert!(matches!(result, Err(AuthError::Validation(_))));
}

#[tokio::test]
async fn test_register_with_invalid_email_formats() {
    let mut mock_repo = MockRepository::new();
    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let invalid_emails = vec![
        "plainaddress",
        "@missinglocal.com",
        "missing@domain",
        "spaces in@local.com",
        "missing.dot@com",
        "double..dot@domain.com",
        "unicode💼@domain.com",
    ];

    for email in invalid_emails {
        let req = RegisterRequest {
            username: "testuser".to_string(),
            email: email.to_string(),
            password: "password123".to_string(),
        };

        let result = service.register(req).await;
        assert!(matches!(result, Err(AuthError::Validation(_))),
            "Email '{}' should be invalid", email);
    }
}

#[tokio::test]
async fn test_register_with_weak_passwords() {
    let mut mock_repo = MockRepository::new();
    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let weak_passwords = vec![
        "short",         // Too short
        "12345678",     // Only numbers
        "abcdefgh",     // Only lowercase
        "        ",     // Only spaces
    ];

    for password in weak_passwords {
        let req = RegisterRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: password.to_string(),
        };

        let result = service.register(req).await;
        assert!(matches!(result, Err(AuthError::Validation(_))),
            "Password '{}' should be invalid", password);
    }
}

#[tokio::test]
async fn test_login_success() {
    let mut mock_repo = MockRepository::new();
    let password = "password123";
    let hash = bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST).unwrap();

    let test_user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        hash,
    );

    // Setup expectations
    mock_repo
        .expect_find_by_username()
        .with(eq("testuser"))
        .returning(move |_| Ok(Some(test_user.clone())));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "testuser".to_string(),
        password: password.to_string(),
    };

    let result = service.login(req).await;
    assert!(result.is_ok());

    let (token, user_response) = result.unwrap();
    assert!(!token.is_empty());
    assert_eq!(user_response.username, "testuser");
}

#[tokio::test]
async fn test_login_invalid_password() {
    let mut mock_repo = MockRepository::new();
    let hash = bcrypt::hash("correct_password".as_bytes(), bcrypt::DEFAULT_COST).unwrap();

    let test_user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        hash,
    );

    // Setup expectations
    mock_repo
        .expect_find_by_username()
        .with(eq("testuser"))
        .returning(move |_| Ok(Some(test_user.clone())));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "testuser".to_string(),
        password: "wrong_password".to_string(),
    };

    let result = service.login(req).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_user_not_found() {
    let mut mock_repo = MockRepository::new();

    // Setup expectations
    mock_repo
        .expect_find_by_username()
        .with(eq("nonexistent"))
        .returning(|_| Ok(None));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "nonexistent".to_string(),
        password: "password123".to_string(),
    };

    let result = service.login(req).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_with_sql_injection_attempt() {
    let mut mock_repo = MockRepository::new();

    // Setup expectations - repository should safely handle the SQL injection attempt
    mock_repo
        .expect_find_by_username()
        .with(eq("' OR '1'='1"))
        .returning(|_| Ok(None));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "' OR '1'='1".to_string(),
        password: "password123".to_string(),
    };

    let result = service.login(req).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_with_very_long_inputs() {
    let mut mock_repo = MockRepository::new();

    // Setup expectations - repository should handle very long inputs gracefully
    mock_repo
        .expect_find_by_username()
        .with(eq(&"a".repeat(1000)))
        .returning(|_| Ok(None));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "a".repeat(1000),
        password: "a".repeat(1000),
    };

    let result = service.login(req).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_with_unicode_username() {
    let mut mock_repo = MockRepository::new();
    let password = "password123";
    let hash = bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST).unwrap();

    let test_user = User::new(
        "测试用户".to_string(),
        "test@example.com".to_string(),
        hash,
    );

    mock_repo
        .expect_find_by_username()
        .with(eq("测试用户"))
        .returning(move |_| Ok(Some(test_user.clone())));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "测试用户".to_string(),
        password: password.to_string(),
    };

    let result = service.login(req).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jwt_token_contains_required_claims() {
    let mut mock_repo = MockRepository::new();
    let password = "password123";
    let hash = bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST).unwrap();

    let test_user = User::new(
        "testuser".to_string(),
        "test@example.com".to_string(),
        hash,
    );
    let user_id = test_user.id;

    mock_repo
        .expect_find_by_username()
        .with(eq("testuser"))
        .returning(move |_| Ok(Some(test_user.clone())));

    let service = AuthService::new(mock_repo, "test_secret".to_string());

    let req = LoginRequest {
        username: "testuser".to_string(),
        password: password.to_string(),
    };

    let result = service.login(req).await;
    assert!(result.is_ok());

    let (token, _) = result.unwrap();

    // Decode and verify token
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &jsonwebtoken::DecodingKey::from_secret("test_secret".as_bytes()),
        &jsonwebtoken::Validation::default()
    ).unwrap();

    assert_eq!(token_data.claims.sub, user_id);
    assert!(token_data.claims.exp > token_data.claims.iat);
    assert!(token_data.claims.exp > chrono::Utc::now().timestamp() as usize);
}

#[test]
fn test_password_hash_verification() {
    let password = "test_password";
    let hash = bcrypt::hash(password.as_bytes(), bcrypt::DEFAULT_COST).unwrap();

    assert!(verify(password.as_bytes(), &hash).unwrap());
    assert!(!verify("wrong_password".as_bytes(), &hash).unwrap());
}

// Add Claims struct needed for JWT verification
#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
    iat: usize,
}
