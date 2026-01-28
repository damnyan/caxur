use crate::domain::permissions::Permission;
use crate::domain::roles::RoleRepository;
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct AttachPermissionUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl AttachPermissionUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(
        &self,
        role_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), AppError> {
        // Check if role exists
        self.repo
            .find_by_id(role_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", role_id)))?;

        // Attach all permissions in a single database query
        self.repo.attach_permissions(role_id, permissions).await?;

        Ok(())
    }
}
