use crate::domain::roles::{NewRole, Role, RoleRepository};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

use uuid::Uuid;

fn default_scope() -> String {
    "ADMINISTRATOR".to_string()
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateRoleRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Role name must be between 3 and 255 characters"
    ))]
    #[schema(example = "Admin", min_length = 3, max_length = 255)]
    pub name: String,
    #[schema(example = "Administrator role with full permissions")]
    pub description: Option<String>,
    #[serde(default = "default_scope")]
    #[schema(example = "ADMINISTRATOR")]
    pub scope: String,
    #[schema(example = "00000000-0000-0000-0000-000000000000")]
    pub group_id: Option<Uuid>,
}

impl CreateRoleRequest {
    /// Custom async validation to check if role name already exists
    pub async fn validate_unique_name(
        &self,
        repo: &Arc<dyn RoleRepository>,
    ) -> Result<(), AppError> {
        if repo
            .find_by_name(&self.name, &self.scope, self.group_id)
            .await?
            .is_some()
        {
            return Err(AppError::ValidationError(
                "Role name already exists".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct CreateRoleUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl CreateRoleUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self, req))]
    pub async fn execute(&self, req: CreateRoleRequest) -> Result<Role, AppError> {
        // Validate unique name
        req.validate_unique_name(&self.repo).await?;

        let new_role = NewRole {
            name: req.name,
            description: req.description,
            scope: req.scope,
            group_id: req.group_id,
        };

        Ok(self.repo.create(new_role).await?)
    }
}
