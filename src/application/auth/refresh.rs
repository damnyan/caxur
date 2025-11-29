use crate::domain::auth::{AuthService, NewRefreshToken, RefreshTokenRepository};
use crate::shared::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::OffsetDateTime;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

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
        let mut hasher = Sha256::new();
        hasher.update(req.refresh_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

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

        // Generate new access token
        let access_token = self
            .auth_service
            .generate_access_token(user_id)
            .map_err(|e| AppError::InternalServerError(e))?;

        // Generate new refresh token
        let new_refresh_token = self
            .auth_service
            .generate_refresh_token(user_id)
            .map_err(|e| AppError::InternalServerError(e))?;

        // Hash new refresh token for storage
        let mut hasher = Sha256::new();
        hasher.update(new_refresh_token.as_bytes());
        let new_token_hash = format!("{:x}", hasher.finalize());

        // Calculate expiration time
        let expires_at =
            OffsetDateTime::now_utc() + time::Duration::seconds(self.refresh_token_expiry);

        // Store new refresh token hash in database
        let refresh_token_entity = NewRefreshToken {
            user_id,
            token_hash: new_token_hash,
            expires_at,
        };

        self.refresh_token_repo
            .create(refresh_token_entity)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        Ok(RefreshTokenResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_expiry,
        })
    }
}
