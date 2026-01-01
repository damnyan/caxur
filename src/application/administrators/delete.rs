use crate::domain::administrators::AdministratorRepository;
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteAdministratorUseCase {
    repo: Arc<dyn AdministratorRepository>,
}

impl DeleteAdministratorUseCase {
    pub fn new(repo: Arc<dyn AdministratorRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<bool, AppError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        Ok(deleted)
    }
}
