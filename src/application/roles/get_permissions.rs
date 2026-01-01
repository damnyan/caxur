use crate::domain::permissions::Permission;
use crate::domain::roles::RoleRepository;
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetRolePermissionsUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl GetRolePermissionsUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, role_id: Uuid) -> Result<Vec<Permission>, AppError> {
        // Check if role exists
        self.repo
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", role_id)))?;

        Ok(self.repo.get_permissions(role_id).await?)
    }
}
