use crate::common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_role() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    let create_request = json!({
        "name": "Test Role",
        "description": "A test role"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    let data = &json["data"];
    assert_eq!(data["type"], "roles");
    assert_eq!(data["attributes"]["name"], "Test Role");
    assert_eq!(data["attributes"]["description"], "A test role");
    assert!(data["attributes"]["createdAt"].is_string());
    assert!(data["attributes"]["updatedAt"].is_string());

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_get_role() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Create a role first
    let create_request = json!({
        "name": "Get Role Test",
        "description": "Description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let role_id = json["data"]["id"].as_str().unwrap();

    // Get the role
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}", role_id))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_get_role_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}", Uuid::new_v4()))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_list_roles() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Create a role
    let create_request = json!({
        "name": "List Role Test",
        "description": "Description"
    });

    app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List roles
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles?page=1&per_page=10")
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
    let data = json["data"].as_array().unwrap();
    assert!(!data.is_empty());

    // Check metadata
    let meta = &json["meta"];
    assert_eq!(meta["page"], 1);
    assert_eq!(meta["perPage"], 10);
    assert_eq!(meta["total"], 1);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_update_role() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Create a role
    let create_request = json!({
        "name": "Update Role Test",
        "description": "Original Description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let role_id = json["data"]["id"].as_str().unwrap();

    // Update role
    let update_request = json!({
        "name": "Updated Role Name",
        "description": "Updated Description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}", role_id))
                .method("PUT")
                .header("content-type", "application/json")
                .body(Body::from(update_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["attributes"]["name"], "Updated Role Name");
    assert_eq!(
        json["data"]["attributes"]["description"],
        "Updated Description"
    );

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_role() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Create a role
    let create_request = json!({
        "name": "Delete Role Test",
        "description": "Description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let role_id = json["data"]["id"].as_str().unwrap();

    // Delete role
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}", role_id))
                .method("DELETE")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check meta
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["meta"]["deleted"], true);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_role_permissions() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Create a role
    let create_request = json!({
        "name": "Permission Role Test",
        "description": "Description"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/roles")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let role_id = json["data"]["id"].as_str().unwrap();

    // 1. Attach permissions
    let attach_request = json!({
        "permissions": ["administrator_management", "role_management"]
    });

    let attach_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}/permissions", role_id))
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(attach_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(attach_response.status(), StatusCode::OK);

    // Check meta
    let body = axum::body::to_bytes(attach_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["meta"]["attached"], true);

    // 2. Get permissions
    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}/permissions", role_id))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    // Note: order is not guaranteed, but standard permissions checks

    // 3. Detach permissions
    let detach_request = json!({
        "permissions": ["administrator_management"]
    });

    let detach_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}/permissions", role_id))
                .method("DELETE")
                .header("content-type", "application/json")
                .body(Body::from(detach_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(detach_response.status(), StatusCode::OK);

    // 4. Verify only one permission remains
    let get_response_2 = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/roles/{}/permissions", role_id))
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(get_response_2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let data = json["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], "role_management");

    common::cleanup_test_db(&pool).await;
}
