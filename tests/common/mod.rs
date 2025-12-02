use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

/// Setup a test database connection
#[allow(dead_code)]
pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    println!("Connecting to test database: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database. Make sure PostgreSQL is running and the database exists.");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Cleanup test database by truncating all tables
#[allow(dead_code)]
pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("TRUNCATE users, refresh_tokens CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test database");
}

/// Generate a test JWT token for authentication
/// This uses the same JWT service as the application
#[allow(dead_code)]
pub fn generate_test_token(user_id: Uuid) -> String {
    use caxur::domain::auth::AuthService;
    use caxur::infrastructure::auth::JwtAuthService;

    // Use the same keys as the application
    let auth_service = JwtAuthService::new(
        "keys/private_key.pem",
        "keys/public_key.pem",
        900,    // 15 minutes
        604800, // 7 days
    )
    .expect("Failed to create auth service for tests");

    auth_service
        .generate_access_token(user_id, "user".to_string())
        .expect("Failed to generate test token")
}
