use crate::domain::administrators::{Administrator, AdministratorRepository};
use crate::shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetAdministratorUseCase {
    repo: Arc<dyn AdministratorRepository>,
}

impl GetAdministratorUseCase {
    pub fn new(repo: Arc<dyn AdministratorRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Option<Administrator>, AppError> {
        let admin = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        Ok(admin)
    }
}
