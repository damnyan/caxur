mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_user() {
    let pool = setup_test_db_or_skip!();

    // Clean up any existing data first
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();

    // If not 201, print the error response
    if status != StatusCode::CREATED {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        println!("Error response: {:#?}", json);
    }

    assert_eq!(status, StatusCode::CREATED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let pool = setup_test_db_or_skip!();
    let app = caxur::presentation::router::app(pool.clone());

    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    // Create first user
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    // Try to create duplicate
    let response2 = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_list_users() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user first
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Extract user ID to generate token
    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    // Generate auth token
    let token = common::generate_test_token(user_uuid);

    // List users with authentication
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("GET")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_get_user() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user first
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Extract user ID from response
    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    // Generate auth token
    let token = common::generate_test_token(user_uuid);

    // Get the user with authentication
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user_id))
                .method("GET")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user to get a valid token
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
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
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    let token = common::generate_test_token(user_uuid);

    let fake_id = "00000000-0000-0000-0000-000000000000";

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", fake_id))
                .method("GET")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_update_user() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user first
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
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
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    let token = common::generate_test_token(user_uuid);

    // Update the user with authentication
    let update_request = json!({
        "username": "updateduser",
        "email": "updated@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_delete_user() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user first
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
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
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    let token = common::generate_test_token(user_uuid);

    // Delete the user with authentication
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user_id))
                .method("DELETE")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_update_user_forbidden() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create two users
    let create_request1 = json!({
        "username": "user1",
        "email": "user1@example.com",
        "password": "password123"
    });

    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
    let user1_id = json1["data"]["id"].as_str().unwrap();
    let user1_uuid = uuid::Uuid::parse_str(user1_id).unwrap();

    let create_request2 = json!({
        "username": "user2",
        "email": "user2@example.com",
        "password": "password123"
    });

    let response2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    let user2_id = json2["data"]["id"].as_str().unwrap();

    // User1 tries to update User2's account (should be forbidden)
    let token = common::generate_test_token(user1_uuid);

    let update_request = json!({
        "username": "hacker",
        "email": "hacker@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user2_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_delete_user_forbidden() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create two users
    let create_request1 = json!({
        "username": "user1",
        "email": "user1@example.com",
        "password": "password123"
    });

    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
        .await
        .unwrap();
    let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
    let user1_id = json1["data"]["id"].as_str().unwrap();
    let user1_uuid = uuid::Uuid::parse_str(user1_id).unwrap();

    let create_request2 = json!({
        "username": "user2",
        "email": "user2@example.com",
        "password": "password123"
    });

    let response2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
    let user2_id = json2["data"]["id"].as_str().unwrap();

    // User1 tries to delete User2's account (should be forbidden)
    let token = common::generate_test_token(user1_uuid);

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user2_id))
                .method("DELETE")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let app = caxur::presentation::router::app(pool.clone());

    // Create a user to get a valid token
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = json["data"]["id"].as_str().unwrap();
    let user_uuid = uuid::Uuid::parse_str(user_id).unwrap();

    let token = common::generate_test_token(user_uuid);

    // Try to delete with the same user ID (should work, but we're testing the not found path)
    // First delete succeeds
    app.clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user_id))
                .method("DELETE")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Second delete should return not found (user already deleted)
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/users/{}", user_id))
                .method("DELETE")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    common::cleanup_test_db(&pool).await;
}
