use crate::application::auth::token_utils::{generate_and_store_tokens, hash_token, TokenResponse};
use crate::domain::auth::{AuthService, RefreshTokenRepository};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

pub type RefreshTokenResponse = TokenResponse;

pub struct RefreshTokenUseCase {
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl RefreshTokenUseCase {
    pub fn new(
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            refresh_token_repo,
            auth_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    pub async fn execute(&self, req: RefreshTokenRequest) -> Result<RefreshTokenResponse, AppError> {
        // Validate the refresh token
        let claims = self
            .auth_service
            .validate_token(&req.refresh_token)
            .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        // Verify token type
        if claims.token_type != "refresh" {
            return Err(AppError::Unauthorized(
                "Invalid token type".to_string(),
            ));
        }

        // Hash the refresh token to find it in the database
        let token_hash = hash_token(&req.refresh_token);

        // Find refresh token in database
        let stored_token = self
            .refresh_token_repo
            .find_by_hash(&token_hash)
            .await
            .map_err(|e| AppError::InternalServerError(e))?
            .ok_or_else(|| AppError::Unauthorized("Refresh token not found or expired".to_string()))?;

        // Parse user ID from claims
        let user_id = claims
            .user_id()
            .map_err(|e| AppError::InternalServerError(e))?;

        // Verify user_id matches
        if stored_token.user_id != user_id {
            return Err(AppError::Unauthorized(
                "Token user mismatch".to_string(),
            ));
        }

        // Delete old refresh token (rotation)
        self.refresh_token_repo
            .delete_by_hash(&token_hash)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        // Generate and store new token pair (preserve user_type)
        generate_and_store_tokens(
            user_id,
            claims.user_type,
            &self.auth_service,
            &self.refresh_token_repo,
            self.access_token_expiry,
            self.refresh_token_expiry,
        )
        .await
    }
}
