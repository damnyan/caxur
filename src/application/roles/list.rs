use crate::domain::roles::{Role, RoleRepository};
use crate::shared::error::AppError;
use std::sync::Arc;

pub struct ListRolesUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl ListRolesUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, per_page: i64, page: i64) -> Result<Vec<Role>, AppError> {
        // Enforce reasonable limits
        let per_page = per_page.clamp(1, 100);
        let page = page.max(1);

        // Calculate offset from page number (page is 1-indexed)
        let offset = (page - 1) * per_page;

        Ok(self.repo.find_all(per_page, offset).await?)
    }
}
