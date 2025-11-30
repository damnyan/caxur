use crate::domain::users::{User, UserRepository};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct PageParams {
    /// Page number (1-indexed)
    #[serde(default = "default_number")]
    #[param(example = 1, minimum = 1)]
    pub number: i64,
    /// Number of items per page
    #[serde(default = "default_size")]
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub size: i64,
    /// Cursor for cursor-based pagination (future use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ListUsersRequest {
    /// Pagination parameters
    #[serde(default)]
    pub page: PageParams,
    /// Sort fields (comma-separated, prefix with - for descending) (future use)
    /// Example: "created_at" or "-created_at,username"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            number: default_number(),
            size: default_size(),
            cursor: None,
        }
    }
}

fn default_number() -> i64 {
    1
}

fn default_size() -> i64 {
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
        let per_page = req.page.size.min(100).max(1);
        let page = req.page.number.max(1);

        // Calculate offset from page number (page is 1-indexed)
        let offset = (page - 1) * per_page;

        self.repo.find_all(per_page, offset).await
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
            page: PageParams {
                number: 1,
                size: 10,
                cursor: None,
            },
            sort: None,
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
            page: PageParams {
                number: 1,
                size: 2,
                cursor: None,
            },
            sort: None,
        };
        let users = use_case.execute(req).await.unwrap();

        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_list_users_pagination() {
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

        // Get page 2 with 2 items per page
        let req = ListUsersRequest {
            page: PageParams {
                number: 2,
                size: 2,
                cursor: None,
            },
            sort: None,
        };
        let users = use_case.execute(req).await.unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "user2");
    }
}
