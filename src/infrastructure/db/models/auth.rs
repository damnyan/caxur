use crate::domain::auth::RefreshToken;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct RefreshTokenDbModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_type: String,
    pub token_hash: String,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl From<RefreshTokenDbModel> for RefreshToken {
    fn from(model: RefreshTokenDbModel) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            user_type: model.user_type,
            token_hash: model.token_hash,
            expires_at: model.expires_at,
            created_at: model.created_at,
        }
    }
}
