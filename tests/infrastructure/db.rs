use caxur::infrastructure::db;

#[tokio::test]
async fn test_create_pool_success() {
    // Set environment variables for predictable test behavior
    unsafe {
        std::env::set_var("DB_MAX_CONNECTIONS", "5");
        std::env::set_var("DB_MIN_CONNECTIONS", "1");
        std::env::set_var("DB_ACQUIRE_TIMEOUT_SECS", "3");
        std::env::set_var("DB_IDLE_TIMEOUT_SECS", "600");
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    let result = db::create_pool(&database_url).await;

    // Skip test if database is not available
    if result.is_err() {
        eprintln!("Skipping test_create_pool_success: database not available");
        return;
    }

    let pool = result.unwrap();

    // Verify the pool works by executing a simple query
    let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();

    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn test_create_pool_invalid_url() {
    let invalid_url = "postgres://invalid:invalid@nonexistent:5432/invalid";

    let result = db::create_pool(invalid_url).await;

    // Should fail to connect
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_pool_invalid_credentials() {
    let invalid_url = "postgres://wronguser:wrongpass@localhost:5432/caxur_test";

    let result = db::create_pool(invalid_url).await;

    // Should fail to authenticate
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_pool_nonexistent_database() {
    let invalid_url = "postgres://postgres:postgres@localhost:5432/nonexistent_db_12345";

    let result = db::create_pool(invalid_url).await;

    // Should fail because database doesn't exist
    assert!(result.is_err());
}

#[tokio::test]
async fn test_pool_connection_limit() {
    // Set environment variables for predictable test behavior
    unsafe {
        std::env::set_var("DB_MAX_CONNECTIONS", "5");
        std::env::set_var("DB_MIN_CONNECTIONS", "1");
        std::env::set_var("DB_ACQUIRE_TIMEOUT_SECS", "3");
        std::env::set_var("DB_IDLE_TIMEOUT_SECS", "600");
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    let pool_result = db::create_pool(&database_url).await;

    // Skip test if database is not available
    if pool_result.is_err() {
        eprintln!("Skipping test_pool_connection_limit: database not available");
        return;
    }

    let pool = pool_result.unwrap();

    // The pool is configured with max_connections(5)
    // We can verify it works by acquiring multiple connections
    let mut connections = vec![];

    // Acquire 5 connections (should succeed)
    for _ in 0..5 {
        let conn = pool.acquire().await.unwrap();
        connections.push(conn);
    }

    // All connections should be acquired successfully
    assert_eq!(connections.len(), 5);

    // Drop connections to return them to the pool
    drop(connections);

    // Should be able to acquire again
    let _conn = pool.acquire().await.unwrap();
}

#[tokio::test]
async fn test_pool_timeout() {
    // Set environment variables for predictable test behavior
    unsafe {
        std::env::set_var("DB_MAX_CONNECTIONS", "5");
        std::env::set_var("DB_MIN_CONNECTIONS", "1");
        std::env::set_var("DB_ACQUIRE_TIMEOUT_SECS", "3");
        std::env::set_var("DB_IDLE_TIMEOUT_SECS", "600");
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    let pool_result = db::create_pool(&database_url).await;

    // Skip test if database is not available
    if pool_result.is_err() {
        eprintln!("Skipping test_pool_timeout: database not available");
        return;
    }

    let pool = pool_result.unwrap();

    // Acquire all connections
    let mut connections = vec![];
    for _ in 0..5 {
        connections.push(pool.acquire().await.unwrap());
    }

    // Try to acquire one more - should timeout (configured for 3 seconds)
    let start = std::time::Instant::now();
    let result = pool.acquire().await;
    let elapsed = start.elapsed();

    // Should fail with timeout
    assert!(result.is_err());

    // Should have waited approximately 3 seconds (with some tolerance)
    assert!(elapsed.as_secs() >= 2 && elapsed.as_secs() <= 4);
}
