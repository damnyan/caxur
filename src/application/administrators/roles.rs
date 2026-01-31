use crate::domain::administrators::AdministratorRepository;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct AttachRoles {
    admin_repo: Arc<dyn AdministratorRepository>,
}

impl AttachRoles {
    pub fn new(admin_repo: Arc<dyn AdministratorRepository>) -> Self {
        Self { admin_repo }
    }

    pub async fn execute(&self, admin_id: Uuid, role_ids: Vec<Uuid>) -> Result<(), anyhow::Error> {
        self.admin_repo.attach_roles(admin_id, role_ids).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DetachRoles {
    admin_repo: Arc<dyn AdministratorRepository>,
}

impl DetachRoles {
    pub fn new(admin_repo: Arc<dyn AdministratorRepository>) -> Self {
        Self { admin_repo }
    }

    pub async fn execute(&self, admin_id: Uuid, role_ids: Vec<Uuid>) -> Result<(), anyhow::Error> {
        self.admin_repo.detach_roles(admin_id, role_ids).await?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, Validate, ToSchema)]
pub struct AttachRolesRequest {
    #[validate(length(min = 1, message = "At least one role must be provided"))]
    pub role_ids: Vec<Uuid>,
}

#[derive(Debug, serde::Deserialize, Validate, ToSchema)]
pub struct DetachRolesRequest {
    #[validate(length(min = 1, message = "At least one role must be provided"))]
    pub role_ids: Vec<Uuid>,
}
