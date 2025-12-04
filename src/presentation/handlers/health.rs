use crate::infrastructure::db::DbPool;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Health check endpoint with database connectivity test
pub async fn health_check(State(pool): State<DbPool>) -> impl IntoResponse {
    // Test database connectivity
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "healthy",
                "database": "connected"
            })),
        ),
        Err(e) => {
            tracing::error!("Database health check failed: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "database": "disconnected"
                })),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;

    #[tokio::test]
    async fn test_health_check_success() {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/caxur_test".to_string()
        });

        // Skip test if database is not available
        let pool = match db::create_pool(&database_url).await {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping health check test: database not available");
                return;
            }
        };

        let response = health_check(State(pool)).await.into_response();

        // Should be OK if database is connected
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        // Create a pool that points to a closed port to simulate failure
        // We use connect_lazy so it doesn't fail immediately on creation
        let pool =
            sqlx::PgPool::connect_lazy("postgres://postgres:postgres@localhost:12345/nonexistent")
                .unwrap();

        let response = health_check(State(pool)).await.into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
