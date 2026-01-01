use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::users::create::{CreateUserRequest, CreateUserUseCase};
use caxur::domain::password::PasswordService;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use serial_test::serial;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_create_user_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateUserUseCase::new(repo, password_service);

    let prefix = uuid::Uuid::new_v4().to_string();
    let req = CreateUserRequest {
        username: format!("create_user_{}", prefix),
        email: format!("create_user_{}@example.com", prefix),
        password: "password123".to_string(),
    };

    let user = use_case.execute(req).await.expect("Failed to create user");

    assert!(user.username.contains(&prefix));
    assert!(user.email.contains(&prefix));
    assert_ne!(user.password_hash, "password123"); // Should be hashed
}

#[tokio::test]
#[serial]
async fn test_create_user_duplicate_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateUserUseCase::new(repo, password_service);

    let prefix = uuid::Uuid::new_v4().to_string();
    let email = format!("dup_email_{}@example.com", prefix);

    // Create first user
    let req1 = CreateUserRequest {
        username: format!("user1_{}", prefix),
        email: email.clone(),
        password: "password123".to_string(),
    };
    use_case
        .execute(req1)
        .await
        .expect("Failed to create first user");

    // Create second user with same email
    let req2 = CreateUserRequest {
        username: format!("user2_{}", prefix),
        email: email,
        password: "password456".to_string(),
    };
    let result = use_case.execute(req2).await;

    assert!(result.is_err());
    // We expect a ValidationError or similar.
    // The exact error type check might be verbose, but checking matches is good.
    // However, `AppError` variants usually wrap internal errors or are specific.
    // The UseCase explicitly returns `AppError::ValidationError("Email already exists")`.
}

use anyhow::anyhow;
use caxur::domain::password::PasswordHashingService;

struct FaultyPasswordHashingService;
impl PasswordHashingService for FaultyPasswordHashingService {
    fn hash_password(&self, _password: &str) -> anyhow::Result<String> {
        Err(anyhow!("Hashing failed"))
    }

    fn verify_password(&self, _password: &str, _hash: &str) -> anyhow::Result<bool> {
        unimplemented!()
    }
}

#[tokio::test]
async fn test_create_user_hash_error() {
    let pool = setup_test_db_or_skip!(); // We need a valid repo, although it won't be called for create if hash fails before?
    // Looking at `CreateUserUseCase::execute`:
    // 1. validate_unique_email (calls repo)
    // 2. hash_password
    // 3. repo.create

    // So we need a repo that works for validation check (find_by_email returns None).

    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(FaultyPasswordHashingService);
    let use_case = CreateUserUseCase::new(repo, password_service);

    let req = CreateUserRequest {
        username: "hash_fail".to_string(),
        email: "hash_fail@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;

    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Hashing failed");
        }
        _ => panic!(
            "Expected InternalServerError(Hashing failed), got {:?}",
            result
        ),
    }
}
