use crate::domain::users::UserRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl DeleteUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        self.repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::NewUser;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_delete_user() {
        let repo = Arc::new(MockUserRepository::default());
        
        let new_user = NewUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
        };
        let created_user = repo.create(new_user).await.unwrap();

        let repo_clone: Arc<dyn UserRepository> = repo.clone();
        let use_case = DeleteUserUseCase::new(repo_clone);
        let deleted = use_case.execute(created_user.id).await.unwrap();

        assert!(deleted);
        
        // Verify user is actually deleted
        let user = repo.find_by_id(created_user.id).await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_user() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = DeleteUserUseCase::new(repo);

        let deleted = use_case.execute(Uuid::new_v4()).await.unwrap();
        assert!(!deleted);
    }
}
