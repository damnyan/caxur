use crate::domain::users::{NewUser, User, UserRepository};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    #[schema(example = "johndoe", min_length = 3)]
    pub username: String,
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "john@example.com")]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "password123", min_length = 6)]
    pub password: String,
}

impl CreateUserRequest {
    /// Custom async validation to check if email already exists
    pub async fn validate_unique_email(
        &self,
        repo: &Arc<dyn UserRepository>,
    ) -> Result<(), AppError> {
        if let Some(_) = repo.find_by_email(&self.email).await? {
            return Err(AppError::ValidationError(
                "Email already exists".to_string(),
            ));
        }
        Ok(())
    }
}

use crate::domain::password::PasswordHashingService;

pub struct CreateUserUseCase {
    repo: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHashingService>,
}

impl CreateUserUseCase {
    pub fn new(
        repo: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHashingService>,
    ) -> Self {
        Self {
            repo,
            password_hasher,
        }
    }

    #[tracing::instrument(skip(self, req))]
    pub async fn execute(&self, req: CreateUserRequest) -> Result<User, AppError> {
        // Validate unique email using custom validator
        req.validate_unique_email(&self.repo).await?;

        // Hash the password using Argon2
        let password_hash = self
            .password_hasher
            .hash_password(&req.password)
            .map_err(|e| AppError::InternalServerError(e))?;

        let new_user = NewUser {
            username: req.username,
            email: req.email,
            password_hash,
        };

        Ok(self.repo.create(new_user).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::password::PasswordService;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_create_user() {
        let repo = Arc::new(MockUserRepository::default());
        let hasher = Arc::new(PasswordService::new());
        let use_case = CreateUserUseCase::new(repo, hasher);

        let req = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let user = use_case.execute(req).await.expect("Failed to create user");

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let repo = Arc::new(MockUserRepository::default());
        let hasher = Arc::new(PasswordService::new());
        let use_case = CreateUserUseCase::new(repo.clone(), hasher);

        // Create first user
        let req1 = CreateUserRequest {
            username: "user1".to_string(),
            email: "duplicate@example.com".to_string(),
            password: "password123".to_string(),
        };
        use_case
            .execute(req1)
            .await
            .expect("Failed to create first user");

        // Try to create second user with same email
        let req2 = CreateUserRequest {
            username: "user2".to_string(),
            email: "duplicate@example.com".to_string(),
            password: "password456".to_string(),
        };
        let result = use_case.execute(req2).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "Email already exists");
            }
            _ => panic!("Expected ValidationError"),
        }
    }
    struct FailingPasswordService;

    #[async_trait::async_trait]
    impl crate::domain::password::PasswordHashingService for FailingPasswordService {
        fn hash_password(&self, _password: &str) -> Result<String, anyhow::Error> {
            Err(anyhow::anyhow!("Hashing error"))
        }
        fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool, anyhow::Error> {
            Err(anyhow::anyhow!("Verification error"))
        }
    }

    #[tokio::test]
    async fn test_create_user_hash_error() {
        let repo = Arc::new(MockUserRepository::default());
        let hasher = Arc::new(FailingPasswordService);
        let use_case = CreateUserUseCase::new(repo, hasher);

        let req = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InternalServerError(e) => assert_eq!(e.to_string(), "Hashing error"),
            _ => panic!("Expected InternalServerError"),
        }
    }
}
