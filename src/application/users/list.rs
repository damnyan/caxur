use crate::domain::users::{User, UserRepository};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};

use crate::shared::pagination::{default_page_number, default_page_size};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct PageParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page_number")]
    #[param(example = 1, minimum = 1)]
    pub number: i64,
    /// Number of items per page
    #[serde(default = "default_page_size")]
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
            number: default_page_number(),
            size: default_page_size(),
            cursor: None,
        }
    }
}

pub struct ListUsersUseCase {
    repo: Arc<dyn UserRepository>,
}

impl ListUsersUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self, req))]
    pub async fn execute(&self, req: ListUsersRequest) -> Result<Vec<User>, anyhow::Error> {
        // Enforce reasonable limits
        let per_page = req.page.size.min(100).max(1);
        let page = req.page.number.max(1);

        // Calculate offset from page number (page is 1-indexed)
        let offset = (page - 1) * per_page;

        self.repo.find_all(per_page, offset).await
    }
}
