use crate::common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    // Try to setup test database
    let pool = match common::setup_test_db().await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test_health_endpoint: database not available");
            return;
        }
    };

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}
