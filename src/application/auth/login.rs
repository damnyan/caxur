use crate::application::auth::token_utils::{generate_and_store_tokens, TokenResponse};
use crate::domain::auth::{AuthService, RefreshTokenRepository};
use crate::domain::password::PasswordService;
use crate::domain::users::UserRepository;
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

pub type LoginResponse = TokenResponse;

pub struct LoginUseCase {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl LoginUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            auth_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    pub async fn execute(&self, req: LoginRequest) -> Result<LoginResponse, AppError> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalServerError(e))?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Verify password
        let is_valid = PasswordService::verify_password(&req.password, &user.password_hash)
            .map_err(|e| AppError::InternalServerError(e))?;

        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        // Generate and store token pair
        generate_and_store_tokens(
            user.id,
            "user".to_string(),
            &self.auth_service,
            &self.refresh_token_repo,
            self.access_token_expiry,
            self.refresh_token_expiry,
        )
        .await
    }
}
