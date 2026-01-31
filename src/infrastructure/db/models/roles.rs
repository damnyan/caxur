use crate::domain::roles::Role;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct RoleDbModel {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub group_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<RoleDbModel> for Role {
    fn from(model: RoleDbModel) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            scope: model.scope,
            group_id: model.group_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
