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
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// Token type: "access" or "refresh"
    #[serde(rename = "type")]
    pub token_type: String,
}

impl Claims {
    pub fn new_access_token(user_id: Uuid, expiry_seconds: i64) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub: user_id.to_string(),
            iat: now,
            exp: now + expiry_seconds,
            token_type: "access".to_string(),
        }
    }

    pub fn new_refresh_token(user_id: Uuid, expiry_seconds: i64) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            sub: user_id.to_string(),
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
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// New refresh token for creation
#[derive(Debug, Clone)]
pub struct NewRefreshToken {
    pub user_id: Uuid,
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
    fn generate_access_token(&self, user_id: Uuid) -> Result<String>;

    /// Generate a refresh token for a user
    fn generate_refresh_token(&self, user_id: Uuid) -> Result<String>;

    /// Validate and decode a token
    fn validate_token(&self, token: &str) -> Result<Claims>;
}
