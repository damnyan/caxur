use crate::domain::users::{User, UserRepository};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ListUsersRequest {
    #[serde(default = "default_limit")]
    #[param(example = 20, maximum = 100)]
    pub limit: i64,
    #[serde(default)]
    #[param(example = 0)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub struct ListUsersUseCase {
    repo: Arc<dyn UserRepository>,
}

impl ListUsersUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, req: ListUsersRequest) -> Result<Vec<User>, anyhow::Error> {
        // Enforce reasonable limits
        let limit = req.limit.min(100).max(1);
        let offset = req.offset.max(0);

        self.repo.find_all(limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::users::NewUser;
    use crate::infrastructure::repositories::mock::MockUserRepository;

    #[tokio::test]
    async fn test_list_users() {
        let repo = Arc::new(MockUserRepository::default());

        // Create multiple users
        for i in 0..3 {
            let new_user = NewUser {
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
                password_hash: "hash123".to_string(),
            };
            repo.create(new_user).await.unwrap();
        }

        let use_case = ListUsersUseCase::new(repo);
        let req = ListUsersRequest {
            limit: 10,
            offset: 0,
        };
        let users = use_case.execute(req).await.unwrap();

        assert_eq!(users.len(), 3);
    }

    #[tokio::test]
    async fn test_list_users_with_limit() {
        let repo = Arc::new(MockUserRepository::default());

        for i in 0..5 {
            let new_user = NewUser {
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
                password_hash: "hash123".to_string(),
            };
            repo.create(new_user).await.unwrap();
        }

        let use_case = ListUsersUseCase::new(repo);
        let req = ListUsersRequest {
            limit: 2,
            offset: 0,
        };
        let users = use_case.execute(req).await.unwrap();

        assert_eq!(users.len(), 2);
    }
}
