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
    use crate::domain::auth::{Claims, RefreshToken};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockAuthService;

    #[async_trait]
    impl AuthService for MockAuthService {
        fn generate_access_token(
            &self,
            _user_id: Uuid,
            _user_type: String,
        ) -> Result<String, anyhow::Error> {
            Ok("access_token".to_string())
        }

        fn generate_refresh_token(
            &self,
            _user_id: Uuid,
            _user_type: String,
        ) -> Result<String, anyhow::Error> {
            Ok("refresh_token".to_string())
        }

        fn validate_token(&self, _token: &str) -> Result<Claims, anyhow::Error> {
            unimplemented!()
        }
    }

    struct MockRefreshTokenRepository {
        tokens: Arc<Mutex<Vec<NewRefreshToken>>>,
    }

    #[async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn create(&self, token: NewRefreshToken) -> Result<RefreshToken, anyhow::Error> {
            self.tokens.lock().unwrap().push(token.clone());
            Ok(RefreshToken {
                id: Uuid::new_v4(),
                user_id: token.user_id,
                user_type: token.user_type,
                token_hash: token.token_hash,
                expires_at: token.expires_at,
                created_at: OffsetDateTime::now_utc(),
            })
        }

        async fn find_by_hash(
            &self,
            _token_hash: &str,
        ) -> Result<Option<RefreshToken>, anyhow::Error> {
            unimplemented!()
        }

        async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64, anyhow::Error> {
            unimplemented!()
        }

        async fn delete_expired(&self) -> Result<u64, anyhow::Error> {
            unimplemented!()
        }

        async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool, anyhow::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn test_hash_token() {
        let token = "test_token";
        let hash = hash_token(token);
        assert_eq!(hash.len(), 64); // SHA-256 hex string length
    }

    #[tokio::test]
    async fn test_generate_and_store_tokens() {
        let auth_service: Arc<dyn AuthService> = Arc::new(MockAuthService);
        let tokens = Arc::new(Mutex::new(Vec::new()));
        let repo: Arc<dyn RefreshTokenRepository> = Arc::new(MockRefreshTokenRepository {
            tokens: tokens.clone(),
        });

        let user_id = Uuid::new_v4();
        let user_type = "user".to_string();
        let access_expiry = 3600;
        let refresh_expiry = 7200;

        let result = generate_and_store_tokens(
            user_id,
            user_type.clone(),
            &auth_service,
            &repo,
            access_expiry,
            refresh_expiry,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.access_token, "access_token");
        assert_eq!(response.refresh_token, "refresh_token");
        assert_eq!(response.expires_in, access_expiry);

        let stored_tokens = tokens.lock().unwrap();
        assert_eq!(stored_tokens.len(), 1);
        assert_eq!(stored_tokens[0].user_id, user_id);
    }
}
