use crate::domain::roles::RoleRepository;
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteRoleUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl DeleteRoleUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("Role with id {} not found", id)));
        }
        Ok(())
    }
}
