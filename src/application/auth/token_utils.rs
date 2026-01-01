use crate::domain::auth::{AuthService, NewRefreshToken, RefreshTokenRepository};
use crate::shared::error::AppError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

/// Common response structure for token operations
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Generate SHA-256 hash of a token string
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate and store a complete token pair (access + refresh tokens)
pub async fn generate_and_store_tokens(
    user_id: Uuid,
    user_type: String,
    auth_service: &Arc<dyn AuthService>,
    refresh_token_repo: &Arc<dyn RefreshTokenRepository>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
) -> Result<TokenResponse, AppError> {
    // Generate access token
    let access_token = auth_service
        .generate_access_token(user_id, user_type.clone())
        .map_err(|e| AppError::InternalServerError(e))?;

    // Generate refresh token
    let refresh_token = auth_service
        .generate_refresh_token(user_id, user_type.clone())
        .map_err(|e| AppError::InternalServerError(e))?;

    // Hash refresh token for storage
    let token_hash = hash_token(&refresh_token);

    // Calculate expiration time
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(refresh_token_expiry);

    // Store refresh token hash in database
    let new_refresh_token = NewRefreshToken {
        user_id,
        user_type,
        token_hash,
        expires_at,
    };

    refresh_token_repo
        .create(new_refresh_token)
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: access_token_expiry,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_token() {
        let token = "test_token";
        let hash = hash_token(token);
        assert_eq!(hash.len(), 64); // SHA-256 hex string length
    }
}
