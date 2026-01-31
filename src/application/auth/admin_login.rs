use crate::application::auth::token_utils::{TokenResponse, generate_and_store_tokens};
use crate::domain::administrators::AdministratorRepository;
use crate::domain::auth::{AuthService, RefreshTokenRepository};
use crate::domain::password::PasswordHashingService;
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct AdminLoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

pub type AdminLoginResponse = TokenResponse;

pub struct AdminLoginUseCase {
    admin_repo: Arc<dyn AdministratorRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    password_service: Arc<dyn PasswordHashingService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl AdminLoginUseCase {
    pub fn new(
        admin_repo: Arc<dyn AdministratorRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        password_service: Arc<dyn PasswordHashingService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            admin_repo,
            refresh_token_repo,
            auth_service,
            password_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    #[tracing::instrument(skip(self, req), fields(email = %req.email))]
    pub async fn execute(&self, req: AdminLoginRequest) -> Result<AdminLoginResponse, AppError> {
        tracing::info!("Attempting admin login for email: {}", req.email);

        // Find administrator by email
        let admin = self
            .admin_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| {
                tracing::error!("Database error finding administrator: {}", e);
                AppError::InternalServerError(e)
            })?
            .ok_or_else(|| {
                tracing::warn!("Administrator not found for email: {}", req.email);
                AppError::Unauthorized("Invalid credentials".to_string())
            })?;

        tracing::info!(
            "Administrator found: {} (ID: {}). Verifying password...",
            admin.email,
            admin.id
        );

        // Verify password
        let valid_password = self
            .password_service
            .verify_password(&req.password, &admin.password_hash)
            .map_err(|e| {
                tracing::error!("Password verification internal error: {}", e);
                AppError::InternalServerError(e)
            })?;

        if !valid_password {
            tracing::warn!(
                "Password verification failed for administrator: {}",
                admin.email
            );
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        tracing::info!("Admin password verified. Generating tokens.");

        // Generate and store token pair with "admin" user type
        generate_and_store_tokens(
            admin.id,
            "admin".to_string(),
            &self.auth_service,
            &self.refresh_token_repo,
            self.access_token_expiry,
            self.refresh_token_expiry,
        )
        .await
    }
}
