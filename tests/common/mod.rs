use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

/// Ensures that the database exists.
pub async fn ensure_test_database_exists(database_url: &str) -> Result<(), sqlx::Error> {
    let options = PgConnectOptions::from_str(database_url)?;
    let database_name = options.get_database().unwrap_or("caxur_test");

    let admin_options = options.clone().database("postgres");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await?;

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

/// Setup a test database connection
#[allow(dead_code)]
pub async fn setup_test_db() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    println!("Connecting to test database: {}", database_url);

    // Ensure database exists
    ensure_test_database_exists(&database_url).await?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

/// Macro to setup test database or skip test if unavailable
#[macro_export]
macro_rules! setup_test_db_or_skip {
    () => {
        match common::setup_test_db().await {
            Ok(pool) => pool,
            Err(_) => {
                eprintln!("Skipping test: database not available");
                return;
            }
        }
    };
}

/// Cleanup test database by truncating all tables
#[allow(dead_code)]
pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query(
        "TRUNCATE users, user_administrators, refresh_tokens, roles, role_permissions CASCADE",
    )
    .execute(pool)
    .await
    .expect("Failed to cleanup test database");
}

use caxur::domain::auth::AuthService;
use caxur::infrastructure::auth::JwtAuthService;
use caxur::infrastructure::state::AppState;
use std::sync::Arc;

pub const TEST_PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgzQ+KuEuDjAghP3/6
0MFOB3poG08f7EBkLt8h0czpsTShRANCAARJRklwE/Tr/osIALEEgegOxArrgT+L
MgWB6ZDIj3woV80aVwPjN2TJC1tzRNeIgJxaVPjLlcvel7450+ct8e8o
-----END PRIVATE KEY-----"#;

pub const TEST_PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAESUZJcBP06/6LCACxBIHoDsQK64E/
izIFgemQyI98KFfNGlcD4zdkyQtbc0TXiICcWlT4y5XL3pe+OdPnLfHvKA==
-----END PUBLIC KEY-----"#;

pub fn create_test_auth_service() -> Arc<JwtAuthService> {
    Arc::new(
        JwtAuthService::new_from_keys(
            TEST_PRIVATE_KEY.as_bytes(),
            TEST_PUBLIC_KEY.as_bytes(),
            900,    // 15 minutes
            604800, // 7 days
        )
        .expect("Failed to create auth service for tests"),
    )
}

pub fn create_test_app_state(pool: PgPool) -> AppState {
    AppState::new(pool, create_test_auth_service())
}

/// Generate a test JWT token for authentication
/// This uses the same JWT service as the application keys
#[allow(dead_code)]
pub fn generate_test_token(user_id: Uuid) -> String {
    let auth_service = create_test_auth_service();
    auth_service
        .generate_access_token(user_id, "user".to_string())
        .expect("Failed to generate test token")
}

pub fn generate_admin_token(user_id: Uuid) -> String {
    let auth_service = create_test_auth_service();
    auth_service
        .generate_access_token(user_id, "admin".to_string())
        .expect("Failed to generate admin token")
}

pub async fn create_admin_with_permissions(pool: &PgPool) -> (Uuid, String) {
    let admin_id = Uuid::new_v4();
    let email = format!("admin_{}@example.com", Uuid::new_v4());

    // 1. Create Administrator
    sqlx::query!(
        "INSERT INTO user_administrators (id, email, password_hash, first_name, last_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
        admin_id,
        email,
        "hash", // Dummy hash
        "Test",
        "Admin"
    )
    .execute(pool)
    .await
    .unwrap();

    // 2. Create Role
    let role_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO roles (id, name, description, scope, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
        role_id,
        format!("Super Admin {}", Uuid::new_v4()),
        "Test Super Admin",
        "ADMINISTRATOR"
    )
    .execute(pool)
    .await
    .unwrap();

    // 3. Assign Wildcard Permission using new schema
    // The permission string is stored in role_permissions table
    sqlx::query!(
        "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)",
        role_id,
        "*"
    )
    .execute(pool)
    .await
    .unwrap();

    // 4. Assign Role to Admin
    sqlx::query!(
        "INSERT INTO administrator_roles (administrator_id, role_id, assigned_at) VALUES ($1, $2, NOW())",
        admin_id,
        role_id
    )
    .execute(pool)
    .await
    .unwrap();

    (admin_id, generate_admin_token(admin_id))
}
