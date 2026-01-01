use crate::common;
use crate::setup_test_db_or_skip;
use anyhow::anyhow;
use caxur::application::users::update::{UpdateUserRequest, UpdateUserUseCase};
use caxur::domain::password::{PasswordHashingService, PasswordService};
use caxur::domain::users::UserRepository;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use caxur::shared::error::AppError;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_update_user_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = UpdateUserUseCase::new(repo.clone(), password_service.clone());

    // Create a user first
    let prefix = Uuid::new_v4().to_string();
    let create_req = caxur::domain::users::NewUser {
        username: format!("user_{}", prefix),
        email: format!("user_{}@example.com", prefix),
        password_hash: "old_hash".to_string(),
    };
    let user = repo
        .create(create_req)
        .await
        .expect("Failed to create user");

    let update_req = UpdateUserRequest {
        username: Some(format!("updated_{}", prefix)),
        email: Some(format!("updated_{}@example.com", prefix)),
        password: Some("newpassword123".to_string()),
    };

    let updated_user = use_case
        .execute(user.id, update_req)
        .await
        .expect("Failed to update user");

    assert_eq!(updated_user.username, format!("updated_{}", prefix));
    assert_eq!(
        updated_user.email,
        format!("updated_{}@example.com", prefix)
    );
    assert!(
        password_service
            .verify_password("newpassword123", &updated_user.password_hash)
            .unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_update_user_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = UpdateUserUseCase::new(repo, password_service);

    let req = UpdateUserRequest {
        username: Some("new_name".to_string()),
        email: None,
        password: None,
    };

    let result = use_case.execute(Uuid::new_v4(), req).await;

    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

#[tokio::test]
#[serial]
async fn test_update_user_email_conflict() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = UpdateUserUseCase::new(repo.clone(), password_service);

    let prefix = Uuid::new_v4().to_string();
    // User 1
    let user1 = repo
        .create(caxur::domain::users::NewUser {
            username: format!("user1_{}", prefix),
            email: format!("user1_{}@example.com", prefix),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();

    // User 2
    let user2 = repo
        .create(caxur::domain::users::NewUser {
            username: format!("user2_{}", prefix),
            email: format!("user2_{}@example.com", prefix),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();

    // Try to update User 2 with User 1's email
    let req = UpdateUserRequest {
        username: None,
        email: Some(user1.email),
        password: None,
    };

    let result = use_case.execute(user2.id, req).await;

    match result {
        Err(AppError::ValidationError(msg)) => {
            assert_eq!(msg, "Email already exists");
        }
        _ => panic!(
            "Expected ValidationError(Email already exists), got {:?}",
            result
        ),
    }
}

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
async fn test_update_user_hash_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(FaultyPasswordHashingService);
    let use_case = UpdateUserUseCase::new(repo.clone(), password_service);

    // Create user
    let user = repo
        .create(caxur::domain::users::NewUser {
            username: "user_hash_test".to_string(),
            email: "user_hash_test@example.com".to_string(),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();

    let req = UpdateUserRequest {
        username: None,
        email: None,
        password: Some("newpassword".to_string()),
    };

    let result = use_case.execute(user.id, req).await;

    match result {
        Err(AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Hashing failed");
        }
        _ => panic!(
            "Expected InternalServerError(Hashing failed), got {:?}",
            result
        ),
    }
}

#[tokio::test]
#[serial]
async fn test_update_user_own_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = UpdateUserUseCase::new(repo.clone(), password_service);

    let prefix = Uuid::new_v4().to_string();
    let user = repo
        .create(caxur::domain::users::NewUser {
            username: format!("user_own_{}", prefix),
            email: format!("user_own_{}@example.com", prefix),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();

    // Update with SAME email
    let req = UpdateUserRequest {
        username: None,
        email: Some(user.email.clone()),
        password: None,
    };

    let result = use_case.execute(user.id, req).await;
    assert!(result.is_ok());
}
