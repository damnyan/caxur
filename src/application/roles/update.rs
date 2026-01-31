use crate::domain::roles::{Role, RoleRepository, UpdateRole};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateRoleRequest {
    #[validate(length(
        min = 3,
        max = 255,
        message = "Role name must be between 3 and 255 characters"
    ))]
    #[schema(example = "Admin", min_length = 3, max_length = 255)]
    pub name: Option<String>,
    #[schema(example = "Administrator role with full permissions")]
    pub description: Option<String>,
}

pub struct UpdateRoleUseCase {
    repo: Arc<dyn RoleRepository>,
}

impl UpdateRoleUseCase {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self, req))]
    pub async fn execute(&self, id: Uuid, req: UpdateRoleRequest) -> Result<Role, AppError> {
        // Check if role exists
        let existing_role = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role with id {} not found", id)))?;

        // Check for duplicate name if name is being updated
        if let Some(ref name) = req.name
            && let Some(duplicate) = self
                .repo
                .find_by_name(name, &existing_role.scope, existing_role.group_id)
                .await?
            && duplicate.id != id
        {
            return Err(AppError::ValidationError(
                "Role name already exists".to_string(),
            ));
        }

        let update = UpdateRole {
            name: req.name,
            description: req.description,
        };

        Ok(self.repo.update(id, update).await?)
    }
}
