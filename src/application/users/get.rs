use crate::domain::users::{User, UserRepository};
use std::sync::Arc;
use uuid::Uuid;

pub struct GetUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl GetUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        self.repo.find_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::NewUser;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_get_user() {
        let repo = Arc::new(MockUserRepository::default());

        // Create a user first
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
        };
        let created_user = repo.create(new_user).await.unwrap();

        let use_case = GetUserUseCase::new(repo);
        let user = use_case.execute(created_user.id).await.unwrap();

        assert!(user.is_some());
        assert_eq!(user.unwrap().username, "testuser");
    }

    #[tokio::test]
    async fn test_get_nonexistent_user() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = GetUserUseCase::new(repo);

        let user = use_case.execute(Uuid::new_v4()).await.unwrap();
        assert!(user.is_none());
    }
}
