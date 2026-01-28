use crate::domain::administrators::{Administrator, AdministratorRepository, NewAdministrator};
use crate::domain::password::PasswordHashingService;
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdministratorRequest {
    #[validate(length(min = 1))]
    pub first_name: String,
    pub middle_name: Option<String>,
    #[validate(length(min = 1))]
    pub last_name: String,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

impl CreateAdministratorRequest {
    /// Custom async validation to check if email already exists
    pub async fn validate_unique_email(
        &self,
        repo: &Arc<dyn AdministratorRepository>,
    ) -> Result<(), AppError> {
        if repo.find_by_email(&self.email).await?.is_some() {
            return Err(AppError::ValidationError(
                "Email already exists".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct CreateAdministratorUseCase {
    repo: Arc<dyn AdministratorRepository>,
    password_service: Arc<dyn PasswordHashingService>,
}

impl CreateAdministratorUseCase {
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
        req: CreateAdministratorRequest,
    ) -> Result<Administrator, AppError> {
        // Check if email already exists
        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::ValidationError(
                "Email already registered".to_string(),
            ));
        }

        let password_hash = self
            .password_service
            .hash_password(&req.password)
            .map_err(AppError::InternalServerError)?;

        let new_admin = NewAdministrator {
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
            .create(new_admin)
            .await
            .map_err(AppError::InternalServerError)?;

        Ok(admin)
    }
}
