use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Administrator {
    pub id: Uuid,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewAdministrator {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAdministrator {
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

#[async_trait]
pub trait AdministratorRepository: Send + Sync {
    async fn create(&self, new_admin: NewAdministrator) -> Result<Administrator, anyhow::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Administrator>, anyhow::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Administrator>, anyhow::Error>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Administrator>, anyhow::Error>;
    async fn count(&self) -> Result<i64, anyhow::Error>;
    async fn update(
        &self,
        id: Uuid,
        update: UpdateAdministrator,
    ) -> Result<Administrator, anyhow::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error>;
}
