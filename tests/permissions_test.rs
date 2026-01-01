mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn test_list_permissions() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/permissions")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify structure and content
    let data = json["data"].as_array().unwrap();
    assert!(!data.is_empty());

    // Check for some known permissions
    let has_admin_management = data
        .iter()
        .any(|p| p["attributes"]["name"] == "administrator_management");
    assert!(has_admin_management);

    common::cleanup_test_db(&pool).await;
}
