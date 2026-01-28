use super::permissions::Permission;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
// ToSchema removed

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn create(&self, new_role: NewRole) -> Result<Role, anyhow::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, anyhow::Error>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, anyhow::Error>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Role>, anyhow::Error>;
    async fn count(&self) -> Result<i64, anyhow::Error>;
    async fn update(&self, id: Uuid, update: UpdateRole) -> Result<Role, anyhow::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error>;

    async fn get_permissions(&self, role_id: Uuid) -> Result<Vec<Permission>, anyhow::Error>;

    // Bulk permission operations for better performance
    async fn attach_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), anyhow::Error>;
    async fn detach_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), anyhow::Error>;
}
