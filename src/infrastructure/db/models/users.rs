use crate::domain::users::User;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserDbModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<UserDbModel> for User {
    fn from(model: UserDbModel) -> Self {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
            password_hash: model.password_hash,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
