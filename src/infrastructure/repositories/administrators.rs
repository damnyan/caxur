use crate::domain::administrators::{
    Administrator, AdministratorRepository, NewAdministrator, UpdateAdministrator,
};
use crate::infrastructure::db::DbPool;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresAdministratorRepository {
    pool: DbPool,
}

impl PostgresAdministratorRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdministratorRepository for PostgresAdministratorRepository {
    #[tracing::instrument(skip(self, new_admin))]
    async fn create(&self, new_admin: NewAdministrator) -> Result<Administrator, anyhow::Error> {
        let admin = sqlx::query_as::<_, Administrator>(
            r#"
            INSERT INTO user_administrators (
                first_name, middle_name, last_name, suffix, contact_number, email, password_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, first_name, middle_name, last_name, suffix, contact_number, email, password_hash, created_at, updated_at
            "#,
        )
        .bind(new_admin.first_name)
        .bind(new_admin.middle_name)
        .bind(new_admin.last_name)
        .bind(new_admin.suffix)
        .bind(new_admin.contact_number)
        .bind(new_admin.email)
        .bind(new_admin.password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(admin)
    }

    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Administrator>, anyhow::Error> {
        let admin = sqlx::query_as::<_, Administrator>(
            r#"
            SELECT id, first_name, middle_name, last_name, suffix, contact_number, email, password_hash, created_at, updated_at
            FROM user_administrators
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(admin)
    }

    #[tracing::instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<Administrator>, anyhow::Error> {
        let admin = sqlx::query_as::<_, Administrator>(
            r#"
            SELECT id, first_name, middle_name, last_name, suffix, contact_number, email, password_hash, created_at, updated_at
            FROM user_administrators
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(admin)
    }

    #[tracing::instrument(skip(self))]
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Administrator>, anyhow::Error> {
        let admins = sqlx::query_as::<_, Administrator>(
            r#"
            SELECT id, first_name, middle_name, last_name, suffix, contact_number, email, password_hash, created_at, updated_at
            FROM user_administrators
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(admins)
    }

    #[tracing::instrument(skip(self))]
    async fn count(&self) -> Result<i64, anyhow::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_administrators
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    #[tracing::instrument(skip(self, update))]
    async fn update(
        &self,
        id: Uuid,
        update: UpdateAdministrator,
    ) -> Result<Administrator, anyhow::Error> {
        let mut query = String::from("UPDATE user_administrators SET ");
        let mut updates = Vec::new();
        let mut param_count = 1;

        if update.first_name.is_some() {
            updates.push(format!("first_name = ${}", param_count));
            param_count += 1;
        }
        if update.middle_name.is_some() {
            updates.push(format!("middle_name = ${}", param_count));
            param_count += 1;
        }
        if update.last_name.is_some() {
            updates.push(format!("last_name = ${}", param_count));
            param_count += 1;
        }
        if update.suffix.is_some() {
            updates.push(format!("suffix = ${}", param_count));
            param_count += 1;
        }
        if update.contact_number.is_some() {
            updates.push(format!("contact_number = ${}", param_count));
            param_count += 1;
        }
        if update.email.is_some() {
            updates.push(format!("email = ${}", param_count));
            param_count += 1;
        }
        if update.password_hash.is_some() {
            updates.push(format!("password_hash = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            // Fetch and return the existing user if no updates are provided,
            // effectively defining a "no-op" update as "return current state"
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Administrator not found"));
        }

        updates.push("updated_at = NOW()".to_string());
        query.push_str(&updates.join(", "));
        query.push_str(&format!(
            " WHERE id = ${} RETURNING id, first_name, middle_name, last_name, suffix, contact_number, email, password_hash, created_at, updated_at",
            param_count
        ));

        let mut query_builder = sqlx::query_as::<_, Administrator>(&query);

        if let Some(first_name) = update.first_name {
            query_builder = query_builder.bind(first_name);
        }
        if let Some(middle_name) = update.middle_name {
            query_builder = query_builder.bind(middle_name);
        }
        if let Some(last_name) = update.last_name {
            query_builder = query_builder.bind(last_name);
        }
        if let Some(suffix) = update.suffix {
            query_builder = query_builder.bind(suffix);
        }
        if let Some(contact_number) = update.contact_number {
            query_builder = query_builder.bind(contact_number);
        }
        if let Some(email) = update.email {
            query_builder = query_builder.bind(email);
        }
        if let Some(password_hash) = update.password_hash {
            query_builder = query_builder.bind(password_hash);
        }
        query_builder = query_builder.bind(id);

        let admin = query_builder.fetch_one(&self.pool).await?;

        Ok(admin)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM user_administrators WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
