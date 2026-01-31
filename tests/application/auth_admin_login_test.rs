use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::auth::admin_login::{AdminLoginRequest, AdminLoginUseCase};
use caxur::domain::administrators::{AdministratorRepository, NewAdministrator};
use caxur::domain::password::PasswordHashingService;
use caxur::infrastructure::password::PasswordService;
use caxur::infrastructure::repositories::administrators::PostgresAdministratorRepository;
use caxur::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use serial_test::serial;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_admin_login_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let admin_repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());

    let prefix = uuid::Uuid::new_v4().to_string();
    let email = format!("admin_login_{}@example.com", prefix);
    let password = "password123";
    let hash = password_service.hash_password(password).unwrap();

    // Create admin
    admin_repo
        .create(NewAdministrator {
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password_hash: hash,
            middle_name: None,
            suffix: None,
            contact_number: None,
        })
        .await
        .expect("Failed to create admin");

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = AdminLoginRequest {
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
async fn test_admin_login_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let admin_repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = AdminLoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    assert!(result.is_err());
    // Since we can't easily check internal error description if it's not public or generic
    // But we know it returns Unauthorized("Invalid credentials")
    // AppError is public, let's verify if we can match it or check string
    // Assuming to_string works or Debug
    let err = result.unwrap_err();
    match err {
        caxur::shared::error::AppError::Unauthorized(msg) => assert_eq!(msg, "Invalid credentials"),
        _ => panic!("Expected Unauthorized error, got {:?}", err),
    }
}

#[tokio::test]
#[serial]
async fn test_admin_login_invalid_password() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let admin_repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());

    let prefix = uuid::Uuid::new_v4().to_string();
    let email = format!("admin_login_wrong_pass_{}@example.com", prefix);
    let password = "password123";
    let hash = password_service.hash_password(password).unwrap();

    admin_repo
        .create(NewAdministrator {
            first_name: "Admin".to_string(),
            last_name: "User".to_string(),
            email: email.clone(),
            password_hash: hash,
            middle_name: None,
            suffix: None,
            contact_number: None,
        })
        .await
        .expect("Failed to create admin");

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = AdminLoginRequest {
        email,
        password: "wrongpassword".to_string(),
    };

    let result = use_case.execute(req).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        caxur::shared::error::AppError::Unauthorized(msg) => {
            assert_eq!(msg, "Invalid email or password")
        }
        _ => panic!("Expected Unauthorized error, got {:?}", err),
    }
}

// Faulty Mocks for Error Handling Tests

use async_trait::async_trait;
use caxur::domain::administrators::{Administrator, UpdateAdministrator};
use caxur::domain::permissions::Permission;

struct FaultyAdministratorRepository;

#[async_trait]
impl AdministratorRepository for FaultyAdministratorRepository {
    async fn create(&self, _new_admin: NewAdministrator) -> Result<Administrator, anyhow::Error> {
        unimplemented!()
    }
    async fn find_by_id(&self, _id: uuid::Uuid) -> Result<Option<Administrator>, anyhow::Error> {
        unimplemented!()
    }
    async fn find_by_email(&self, _email: &str) -> Result<Option<Administrator>, anyhow::Error> {
        Err(anyhow::anyhow!("Database failure"))
    }
    async fn find_all(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<Administrator>, anyhow::Error> {
        unimplemented!()
    }
    async fn count(&self) -> Result<i64, anyhow::Error> {
        unimplemented!()
    }
    async fn update(
        &self,
        _id: uuid::Uuid,
        _update: UpdateAdministrator,
    ) -> Result<Administrator, anyhow::Error> {
        unimplemented!()
    }
    async fn delete(&self, _id: uuid::Uuid) -> Result<bool, anyhow::Error> {
        unimplemented!()
    }
    async fn attach_roles(
        &self,
        _admin_id: uuid::Uuid,
        _role_ids: Vec<uuid::Uuid>,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
    async fn detach_roles(
        &self,
        _admin_id: uuid::Uuid,
        _role_ids: Vec<uuid::Uuid>,
    ) -> Result<(), anyhow::Error> {
        unimplemented!()
    }
    async fn get_permissions(
        &self,
        _admin_id: uuid::Uuid,
    ) -> Result<Vec<Permission>, anyhow::Error> {
        unimplemented!()
    }
}

struct FaultyPasswordService;

impl PasswordHashingService for FaultyPasswordService {
    fn hash_password(&self, _password: &str) -> Result<String, anyhow::Error> {
        Ok("hashed".to_string())
    }

    fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool, anyhow::Error> {
        Err(anyhow::anyhow!("Hashing internal error"))
    }
}

#[tokio::test]
async fn test_admin_login_repo_error() {
    let auth_service = common::create_test_auth_service();
    let password_service = Arc::new(PasswordService::new());
    let admin_repo = Arc::new(FaultyAdministratorRepository);
    // Refresh repo unused if admin lookup fails
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(
        sqlx::PgPool::connect_lazy("postgres://localhost:5432/dummy").unwrap(),
    ));

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_repo,
        auth_service,
        password_service,
        3600,
        7200,
    );

    let req = AdminLoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Database failure");
        }
        _ => panic!("Expected InternalServerError, got {:?}", result),
    }
}

#[tokio::test]
#[serial]
async fn test_admin_login_password_service_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let admin_repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();
    let faulty_password_service = Arc::new(FaultyPasswordService);

    // Create a user so find_by_email succeeds
    let repo_clone = admin_repo.clone();
    let prefix = uuid::Uuid::new_v4();
    let email = format!("faulty_admin_{}@example.com", prefix);

    // We need to create a real admin so the repo returns one
    repo_clone
        .create(NewAdministrator {
            first_name: "Faulty".to_string(),
            last_name: "Admin".to_string(),
            email: email.clone(),
            password_hash: "hash".to_string(),
            middle_name: None,
            suffix: None,
            contact_number: None,
        })
        .await
        .unwrap();

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_repo,
        auth_service,
        faulty_password_service,
        3600,
        7200,
    );

    let req = AdminLoginRequest {
        email,
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Hashing internal error");
        }
        _ => panic!("Expected InternalServerError, got {:?}", result),
    }
}
