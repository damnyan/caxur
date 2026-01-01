use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::auth::login::{LoginRequest, LoginUseCase};
use caxur::domain::password::{PasswordHashingService, PasswordService};
use caxur::domain::users::{NewUser, UserRepository};
use caxur::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use serial_test::serial;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_login_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());

    let prefix = uuid::Uuid::new_v4().to_string();
    let email = format!("login_{}@example.com", prefix);
    let password = "password123";
    let hash = password_service.hash_password(password).unwrap();

    // Create user manually in repo
    user_repo
        .create(NewUser {
            username: format!("login_{}", prefix),
            email: email.clone(),
            password_hash: hash,
        })
        .await
        .expect("Failed to create user");

    let use_case = LoginUseCase::new(
        user_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600, // 1 hour
        7200, // 2 hours
    );

    let req = LoginRequest {
        email,
        password: password.to_string(),
    };

    let response = use_case.execute(req).await.expect("Login failed");

    assert!(!response.access_token.is_empty());
    assert!(!response.refresh_token.is_empty());
    assert_eq!(response.token_type, "Bearer");
}

#[tokio::test]
#[serial]
async fn test_login_invalid_credentials() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());

    let use_case = LoginUseCase::new(
        user_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    assert!(result.is_err());
}
// Faulty Mocks for Error Handling Tests

use async_trait::async_trait;
use caxur::domain::users::User;

struct FaultyUserRepository;

#[async_trait]
impl UserRepository for FaultyUserRepository {
    async fn create(&self, _user: NewUser) -> Result<User, anyhow::Error> {
        unimplemented!()
    }

    async fn find_by_email(&self, _email: &str) -> Result<Option<User>, anyhow::Error> {
        Err(anyhow::anyhow!("Database failure"))
    }

    async fn find_by_id(&self, _id: uuid::Uuid) -> Result<Option<User>, anyhow::Error> {
        unimplemented!()
    }

    async fn update(
        &self,
        _id: uuid::Uuid,
        _data: caxur::domain::users::UpdateUser,
    ) -> Result<User, anyhow::Error> {
        unimplemented!()
    }

    async fn delete(&self, _id: uuid::Uuid) -> Result<bool, anyhow::Error> {
        unimplemented!()
    }

    async fn count(&self) -> Result<i64, anyhow::Error> {
        unimplemented!()
    }

    async fn find_all(&self, _limit: i64, _offset: i64) -> Result<Vec<User>, anyhow::Error> {
        unimplemented!()
    }
}

struct FaultyPasswordService;

impl PasswordHashingService for FaultyPasswordService {
    fn hash_password(&self, _password: &str) -> Result<String, anyhow::Error> {
        Ok("hashed".to_string())
    }

    fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool, anyhow::Error> {
        Err(anyhow::anyhow!("Hashing failure"))
    }
}

#[tokio::test]
async fn test_login_user_repo_error() {
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());
    // We can pass dummy repos for ones we don't expect to be called fully or used
    let user_repo = Arc::new(FaultyUserRepository);
    // Refresh repo not used if user search fails
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(
        sqlx::PgPool::connect_lazy("postgres://localhost:5432/dummy").unwrap(),
    ));

    let use_case = LoginUseCase::new(
        user_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    assert!(matches!(
        result,
        Err(caxur::shared::error::AppError::InternalServerError(_))
    ));
}

#[tokio::test]
#[serial]
async fn test_login_password_service_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let faulty_password_service = Arc::new(FaultyPasswordService);

    // Create a user so find_by_email succeeds
    let repo_clone = user_repo.clone();
    let prefix = uuid::Uuid::new_v4();
    let email = format!("faulty_{}@example.com", prefix);
    repo_clone
        .create(NewUser {
            username: format!("faulty_{}", prefix),
            email: email.clone(),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();

    let use_case = LoginUseCase::new(
        user_repo,
        refresh_repo,
        auth_service,
        faulty_password_service,
        3600,
        7200,
    );

    let req = LoginRequest {
        email,
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    assert!(matches!(
        result,
        Err(caxur::shared::error::AppError::InternalServerError(_))
    ));
}
