use crate::domain::roles::{Role, RoleRepository};
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetRoleUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl GetRoleUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid) -> Result<Role, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", id)))
    }
}
