use crate::domain::permissions::Permission;
use crate::domain::roles::{NewRole, Role, RoleRepository, UpdateRole};
use crate::infrastructure::db::DbPool;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresRoleRepository {
    pool: DbPool,
}

impl PostgresRoleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleRepository for PostgresRoleRepository {
    #[tracing::instrument(skip(self, new_role))]
    async fn create(&self, new_role: NewRole) -> Result<Role, anyhow::Error> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (name, description)
            VALUES ($1, $2)
            RETURNING id, name, description, created_at, updated_at
            "#,
        )
        .bind(new_role.name)
        .bind(new_role.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(role)
    }

    #[tracing::instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, anyhow::Error> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    #[tracing::instrument(skip(self))]
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, anyhow::Error> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM roles
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(role)
    }

    #[tracing::instrument(skip(self))]
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Role>, anyhow::Error> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM roles
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    #[tracing::instrument(skip(self))]
    async fn count(&self) -> Result<i64, anyhow::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM roles
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    #[tracing::instrument(skip(self, update))]
    async fn update(&self, id: Uuid, update: UpdateRole) -> Result<Role, anyhow::Error> {
        // Build dynamic query based on what fields are being updated
        let mut query = String::from("UPDATE roles SET ");
        let mut updates = Vec::new();
        let mut param_count = 1;

        if update.name.is_some() {
            updates.push(format!("name = ${}", param_count));
            param_count += 1;
        }
        if update.description.is_some() {
            updates.push(format!("description = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Err(anyhow::anyhow!("No fields to update"));
        }

        updates.push("updated_at = NOW()".to_string());
        query.push_str(&updates.join(", "));
        query.push_str(&format!(
            " WHERE id = ${} RETURNING id, name, description, created_at, updated_at",
            param_count
        ));

        let mut query_builder = sqlx::query_as::<_, Role>(&query);

        if let Some(name) = update.name {
            query_builder = query_builder.bind(name);
        }
        if let Some(description) = update.description {
            query_builder = query_builder.bind(description);
        }
        query_builder = query_builder.bind(id);

        let role = query_builder.fetch_one(&self.pool).await?;

        Ok(role)
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    #[tracing::instrument(skip(self))]
    async fn get_permissions(&self, role_id: Uuid) -> Result<Vec<Permission>, anyhow::Error> {
        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT permission FROM role_permissions WHERE role_id = $1",
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        permissions
            .into_iter()
            .map(|p| p.parse().map_err(|e: String| anyhow::anyhow!(e)))
            .collect()
    }

    async fn attach_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), anyhow::Error> {
        if permissions.is_empty() {
            return Ok(());
        }

        // Build bulk INSERT query with ON CONFLICT DO NOTHING to handle duplicates
        let mut query_builder =
            sqlx::QueryBuilder::new("INSERT INTO role_permissions (role_id, permission) ");

        query_builder.push_values(permissions, |mut b, permission| {
            b.push_bind(role_id).push_bind(permission.to_string());
        });

        query_builder.push(" ON CONFLICT (role_id, permission) DO NOTHING");

        query_builder.build().execute(&self.pool).await?;

        Ok(())
    }

    async fn detach_permissions(
        &self,
        role_id: Uuid,
        permissions: Vec<Permission>,
    ) -> Result<(), anyhow::Error> {
        if permissions.is_empty() {
            return Ok(());
        }

        // Convert permissions to strings for the query
        let permission_strings: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();

        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission = ANY($2)")
            .bind(role_id)
            .bind(&permission_strings)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::*;
    use crate::infrastructure::db::create_pool;
    use sqlx::ConnectOptions;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use std::str::FromStr;

    /// Ensures that the database exists.
    pub async fn ensure_test_database_exists(database_url: &str) -> Result<(), sqlx::Error> {
        let options = PgConnectOptions::from_str(database_url)?;
        let database_name = options.get_database().unwrap_or("caxur_test");

        // Connect to the default 'postgres' database to check/create the target database
        let admin_options = options.clone().database("postgres");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(admin_options)
            .await?;

        // Check if database exists
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(database_name)
                .fetch_one(&pool)
                .await?;

        if !exists {
            println!("Database {} does not exist. Creating...", database_name);
            let query = format!("CREATE DATABASE \"{}\"", database_name);
            sqlx::query(&query).execute(&pool).await?;
            println!("Database {} created successfully.", database_name);
        }

        Ok(())
    }

    async fn setup_test_db() -> DbPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/caxur_test".to_string()
        });

        ensure_test_database_exists(&database_url).await.unwrap();

        let pool = create_pool(&database_url).await.unwrap();

        // Run migrations
        sqlx::migrate!().run(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_role() {
        let pool = setup_test_db().await;
        let repo = PostgresRoleRepository::new(pool.clone());

        let new_role = NewRole {
            name: format!("test_role_{}", uuid::Uuid::new_v4()),
            description: Some("Test role description".to_string()),
        };

        let result = repo.create(new_role.clone()).await;
        assert!(result.is_ok());

        let role = result.unwrap();
        assert_eq!(role.name, new_role.name);
        assert_eq!(role.description, new_role.description);

        // Cleanup
        repo.delete(role.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_find_by_name() {
        let pool = setup_test_db().await;
        let repo = PostgresRoleRepository::new(pool.clone());

        let new_role = NewRole {
            name: format!("test_role_{}", uuid::Uuid::new_v4()),
            description: None,
        };

        let created = repo.create(new_role.clone()).await.unwrap();
        let found = repo.find_by_name(&new_role.name).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_attach_and_get_permissions() {
        let pool = setup_test_db().await;
        let repo = PostgresRoleRepository::new(pool.clone());

        let new_role = NewRole {
            name: format!("test_role_{}", uuid::Uuid::new_v4()),
            description: None,
        };

        let role = repo.create(new_role).await.unwrap();

        // Attach permissions
        repo.attach_permissions(
            role.id,
            vec![
                Permission::AdministratorManagement,
                Permission::RoleManagement,
            ],
        )
        .await
        .unwrap();

        // Get permissions
        let permissions = repo.get_permissions(role.id).await.unwrap();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&Permission::AdministratorManagement));
        assert!(permissions.contains(&Permission::RoleManagement));

        // Cleanup
        repo.delete(role.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_detach_permission() {
        let pool = setup_test_db().await;
        let repo = PostgresRoleRepository::new(pool.clone());

        let new_role = NewRole {
            name: format!("test_role_{}", uuid::Uuid::new_v4()),
            description: None,
        };

        let role = repo.create(new_role).await.unwrap();

        // Attach and detach
        repo.attach_permissions(role.id, vec![Permission::AdministratorManagement])
            .await
            .unwrap();
        repo.detach_permissions(role.id, vec![Permission::AdministratorManagement])
            .await
            .unwrap();

        let permissions = repo.get_permissions(role.id).await.unwrap();
        assert_eq!(permissions.len(), 0);

        // Cleanup
        repo.delete(role.id).await.unwrap();
    }
}
