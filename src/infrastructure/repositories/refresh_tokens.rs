use crate::domain::auth::{NewRefreshToken, RefreshToken, RefreshTokenRepository};
use crate::infrastructure::db::DbPool;
use crate::infrastructure::db::models::auth::RefreshTokenDbModel;
use anyhow::Result;
use async_trait::async_trait;
use sqlx;
use uuid::Uuid;

pub struct PostgresRefreshTokenRepository {
    pool: DbPool,
}

impl PostgresRefreshTokenRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresRefreshTokenRepository {
    async fn create(&self, token: NewRefreshToken) -> Result<RefreshToken> {
        let token_db = sqlx::query_as::<_, RefreshTokenDbModel>(
            r#"
            INSERT INTO refresh_tokens (user_id, user_type, token_hash, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, user_type, token_hash, expires_at, created_at
            "#,
        )
        .bind(token.user_id)
        .bind(&token.user_type)
        .bind(&token.token_hash)
        .bind(token.expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(token_db.into())
    }

    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        let token_db = sqlx::query_as::<_, RefreshTokenDbModel>(
            r#"
            SELECT id, user_id, user_type, token_hash, expires_at, created_at
            FROM refresh_tokens
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(token_db.map(|t| t.into()))
    }

    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at <= NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn delete_by_hash(&self, token_hash: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
