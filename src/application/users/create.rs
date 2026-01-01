use crate::domain::password::PasswordHashingService;
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
