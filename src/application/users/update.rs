use crate::domain::password::PasswordHashingService;
use crate::domain::users::{UpdateUser, User, UserRepository};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    #[schema(example = "johndoe_updated", min_length = 3)]
    pub username: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    #[schema(example = "newemail@example.com")]
    pub email: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "newpassword123", min_length = 6)]
    pub password: Option<String>,
}

impl UpdateUserRequest {
    /// Custom async validation to check if email already exists (excluding current user)
    pub async fn validate_unique_email(
        &self,
        repo: &Arc<dyn UserRepository>,
        current_user_id: Uuid,
    ) -> Result<(), AppError> {
        if let Some(email) = &self.email {
            if let Some(existing_user) = repo.find_by_email(email).await? {
                // Only error if the email belongs to a different user
                if existing_user.id != current_user_id {
                    return Err(AppError::ValidationError(
                        "Email already exists".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

pub struct UpdateUserUseCase {
    repo: Arc<dyn UserRepository>,
    password_hasher: Arc<dyn PasswordHashingService>,
}

impl UpdateUserUseCase {
    pub fn new(
        repo: Arc<dyn UserRepository>,
        password_hasher: Arc<dyn PasswordHashingService>,
    ) -> Self {
        Self {
            repo,
            password_hasher,
        }
    }

    pub async fn execute(&self, id: Uuid, req: UpdateUserRequest) -> Result<User, AppError> {
        // Check if user exists
        let existing = self.repo.find_by_id(id).await?;
        if existing.is_none() {
            return Err(AppError::NotFound(format!("User with id {} not found", id)));
        }

        // Validate unique email using custom validator (ignoring current user)
        req.validate_unique_email(&self.repo, id).await?;

        // Hash the password if it's being updated
        let password_hash = if let Some(password) = req.password {
            Some(
                self.password_hasher
                    .hash_password(&password)
                    .map_err(|e| AppError::InternalServerError(e))?,
            )
        } else {
            None
        };

        let update = UpdateUser {
            username: req.username,
            email: req.email,
            password_hash,
        };

        Ok(self.repo.update(id, update).await?)
    }
}
