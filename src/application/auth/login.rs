use crate::domain::auth::{AuthService, NewRefreshToken, RefreshTokenRepository};
use crate::domain::password::PasswordService;
use crate::domain::users::UserRepository;
use crate::shared::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

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

        // Generate access token
        let access_token = self
            .auth_service
            .generate_access_token(user.id)
            .map_err(|e| AppError::InternalServerError(e))?;

        // Generate refresh token
        let refresh_token = self
            .auth_service
            .generate_refresh_token(user.id)
            .map_err(|e| AppError::InternalServerError(e))?;

        // Hash refresh token for storage (using SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Calculate expiration time
        let expires_at =
            OffsetDateTime::now_utc() + time::Duration::seconds(self.refresh_token_expiry);

        // Store refresh token hash in database
        let new_refresh_token = NewRefreshToken {
            user_id: user.id,
            token_hash,
            expires_at,
        };

        self.refresh_token_repo
            .create(new_refresh_token)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry,
        })
    }
}
