use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    #[schema(ignore)]
    pub password_hash: String,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String, example = "2025-11-29T10:00:00Z")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String, example = "2025-11-29T10:00:00Z")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, new_user: NewUser) -> Result<User, anyhow::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, anyhow::Error>;
    async fn update(&self, id: Uuid, update: UpdateUser) -> Result<User, anyhow::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error>;
}
