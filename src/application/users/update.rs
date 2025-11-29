use crate::domain::users::{UpdateUser, User, UserRepository};
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

pub struct UpdateUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl UpdateUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid, req: UpdateUserRequest) -> Result<User, anyhow::Error> {
        // Check if user exists
        let existing = self.repo.find_by_id(id).await?;
        if existing.is_none() {
            return Err(anyhow::anyhow!("User not found"));
        }

        let update = UpdateUser {
            username: req.username,
            email: req.email,
            password_hash: req.password, // In real app, hash the password here
        };

        self.repo.update(id, update).await
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
}
