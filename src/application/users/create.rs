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

pub struct CreateUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, req: CreateUserRequest) -> Result<User, AppError> {
        // Check if email already exists
        if let Some(_) = self.repo.find_by_email(&req.email).await? {
            return Err(AppError::ValidationError(
                "Email already exists".to_string(),
            ));
        }

        // In a real app, you'd hash the password here
        let new_user = NewUser {
            username: req.username,
            email: req.email,
            password_hash: req.password, // Placeholder for hashing
        };

        Ok(self.repo.create(new_user).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_create_user() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = CreateUserUseCase::new(repo);

        let req = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let user = use_case.execute(req).await.expect("Failed to create user");

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }
}
