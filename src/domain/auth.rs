use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User type: "user", "admin", "merchant", etc.
    pub user_type: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Token type: "access" or "refresh"
    #[serde(rename = "type")]
    pub token_type: String,
}

impl Claims {
    pub fn new_access_token(user_id: Uuid, user_type: String, expiry_seconds: i64) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub: user_id.to_string(),
            user_type,
            iat: now,
            exp: now + expiry_seconds,
            token_type: "access".to_string(),
        }
    }

    pub fn new_refresh_token(user_id: Uuid, user_type: String, expiry_seconds: i64) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub: user_id.to_string(),
            user_type,
            iat: now,
            exp: now + expiry_seconds,
            token_type: "refresh".to_string(),
        }
    }

    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|e| anyhow::anyhow!("Invalid user ID in claims: {}", e))
    }
}

/// Refresh token entity
#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_type: String,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// New refresh token for creation
#[derive(Debug, Clone)]
pub struct NewRefreshToken {
    pub user_id: Uuid,
    pub user_type: String,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
}

/// Repository trait for refresh tokens
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// Create a new refresh token
    async fn create(&self, token: NewRefreshToken) -> Result<RefreshToken>;

    /// Find a refresh token by its hash
    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>>;

    /// Delete all refresh tokens for a user
    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64>;

    /// Delete expired refresh tokens
    async fn delete_expired(&self) -> Result<u64>;

    /// Delete a specific refresh token by hash
    async fn delete_by_hash(&self, token_hash: &str) -> Result<bool>;
}

/// Auth service trait for JWT operations
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Generate an access token for a user
    fn generate_access_token(&self, user_id: Uuid, user_type: String) -> Result<String>;

    /// Generate a refresh token for a user
    fn generate_refresh_token(&self, user_id: Uuid, user_type: String) -> Result<String>;

    /// Validate and decode a token
    fn validate_token(&self, token: &str) -> Result<Claims>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_access_token() {
        let user_id = Uuid::new_v4();
        let user_type = "user".to_string();
        let expiry_seconds = 3600;

        let claims = Claims::new_access_token(user_id, user_type.clone(), expiry_seconds);

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.user_type, user_type);
        assert_eq!(claims.token_type, "access");
        assert!(claims.iat > 0);
        assert_eq!(claims.exp, claims.iat + expiry_seconds);
    }

    #[test]
    fn test_new_refresh_token() {
        let user_id = Uuid::new_v4();
        let user_type = "admin".to_string();
        let expiry_seconds = 7200;

        let claims = Claims::new_refresh_token(user_id, user_type.clone(), expiry_seconds);

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.user_type, user_type);
        assert_eq!(claims.token_type, "refresh");
        assert!(claims.iat > 0);
        assert_eq!(claims.exp, claims.iat + expiry_seconds);
    }

    #[test]
    fn test_user_id_valid() {
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            user_type: "user".to_string(),
            iat: 0,
            exp: 0,
            token_type: "access".to_string(),
        };

        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[test]
    fn test_user_id_invalid() {
        let claims = Claims {
            sub: "invalid-uuid".to_string(),
            user_type: "user".to_string(),
            iat: 0,
            exp: 0,
            token_type: "access".to_string(),
        };

        assert!(claims.user_id().is_err());
    }
}
