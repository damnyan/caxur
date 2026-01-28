use crate::domain::administrators::{Administrator, AdministratorRepository, UpdateAdministrator};
use crate::domain::password::PasswordHashingService;
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdministratorRequest {
    #[validate(length(min = 1))]
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    #[validate(length(min = 1))]
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[schema(example = "newpassword123", min_length = 6)]
    pub password: Option<String>,
}

impl UpdateAdministratorRequest {
    /// Custom async validation to check if email already exists (excluding current user)
    pub async fn validate_unique_email(
        &self,
        repo: &Arc<dyn AdministratorRepository>,
        current_user_id: Uuid,
    ) -> Result<(), AppError> {
        if let Some(email) = &self.email
            && let Some(existing_user) = repo.find_by_email(email).await?
            && existing_user.id != current_user_id
        {
            return Err(AppError::ValidationError(
                "Email already registered".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct UpdateAdministratorUseCase {
    repo: Arc<dyn AdministratorRepository>,
    password_service: Arc<dyn PasswordHashingService>,
}

impl UpdateAdministratorUseCase {
    pub fn new(
        repo: Arc<dyn AdministratorRepository>,
        password_service: Arc<dyn PasswordHashingService>,
    ) -> Self {
        Self {
            repo,
            password_service,
        }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        req: UpdateAdministratorRequest,
    ) -> Result<Administrator, AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await
            .map_err(AppError::InternalServerError)?;

        if existing.is_none() {
            return Err(AppError::NotFound(format!(
                "Administrator with id {} not found",
                id
            )));
        }

        // Validate unique email using custom validator (ignoring current user)
        req.validate_unique_email(&self.repo, id).await?;

        let password_hash = if let Some(password) = req.password {
            Some(
                self.password_service
                    .hash_password(&password)
                    .map_err(AppError::InternalServerError)?,
            )
        } else {
            None
        };

        let update_struct = UpdateAdministrator {
            first_name: req.first_name,
            middle_name: req.middle_name,
            last_name: req.last_name,
            suffix: req.suffix,
            contact_number: req.contact_number,
            email: req.email,
            password_hash,
        };

        let admin = self
            .repo
            .update(id, update_struct)
            .await
            .map_err(AppError::InternalServerError)?;

        Ok(admin)
    }
}
