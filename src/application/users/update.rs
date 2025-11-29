use crate::domain::password::PasswordService;
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
}

impl UpdateUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid, req: UpdateUserRequest) -> Result<User, AppError> {
        // Check if user exists
        let existing = self.repo.find_by_id(id).await?;
        if existing.is_none() {
            return Err(AppError::NotFound);
        }

        // Validate unique email using custom validator (ignoring current user)
        req.validate_unique_email(&self.repo, id).await?;

        // Hash the password if it's being updated
        let password_hash = if let Some(password) = req.password {
            Some(
                PasswordService::hash_password(&password)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::NewUser;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_update_user() {
        let repo = Arc::new(MockUserRepository::default());

        let new_user = NewUser {
            username: "oldname".to_string(),
            email: "old@example.com".to_string(),
            password_hash: "hash123".to_string(),
        };
        let created_user = repo.create(new_user).await.unwrap();

        let use_case = UpdateUserUseCase::new(repo);
        let req = UpdateUserRequest {
            username: Some("newname".to_string()),
            email: None,
            password: None,
        };

        let updated_user = use_case.execute(created_user.id, req).await.unwrap();

        assert_eq!(updated_user.username, "newname");
        assert_eq!(updated_user.email, "old@example.com");
    }

    #[tokio::test]
    async fn test_update_nonexistent_user() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = UpdateUserUseCase::new(repo);

        let req = UpdateUserRequest {
            username: Some("newname".to_string()),
            email: None,
            password: None,
        };

        let result = use_case.execute(Uuid::new_v4(), req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_user_duplicate_email() {
        let repo = Arc::new(MockUserRepository::default());

        // Create two users
        let _user1 = repo
            .create(NewUser {
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
                password_hash: "hash1".to_string(),
            })
            .await
            .unwrap();

        let user2 = repo
            .create(NewUser {
                username: "user2".to_string(),
                email: "user2@example.com".to_string(),
                password_hash: "hash2".to_string(),
            })
            .await
            .unwrap();

        let use_case = UpdateUserUseCase::new(repo);

        // Try to update user2 with user1's email
        let req = UpdateUserRequest {
            username: None,
            email: Some("user1@example.com".to_string()),
            password: None,
        };

        let result = use_case.execute(user2.id, req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError(msg) => {
                assert_eq!(msg, "Email already exists");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_update_user_same_email() {
        let repo = Arc::new(MockUserRepository::default());

        let user = repo
            .create(NewUser {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
            })
            .await
            .unwrap();

        let use_case = UpdateUserUseCase::new(repo);

        // Update user with their own email should succeed
        let req = UpdateUserRequest {
            username: Some("newname".to_string()),
            email: Some("test@example.com".to_string()),
            password: None,
        };

        let result = use_case.execute(user.id, req).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.username, "newname");
        assert_eq!(updated.email, "test@example.com");
    }
}
