use crate::domain::administrators::Administrator;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AdministratorDbModel {
    pub id: Uuid,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<AdministratorDbModel> for Administrator {
    fn from(model: AdministratorDbModel) -> Self {
        Self {
            id: model.id,
            first_name: model.first_name,
            middle_name: model.middle_name,
            last_name: model.last_name,
            suffix: model.suffix,
            contact_number: model.contact_number,
            email: model.email,
            password_hash: model.password_hash,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
