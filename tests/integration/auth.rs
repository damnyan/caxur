use crate::common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn test_login_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // First create a user
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    app.clone()
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

    // Now login
    let login_request = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify response contains tokens
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"]["attributes"]["accessToken"].is_string());
    assert!(json["data"]["attributes"]["refreshToken"].is_string());
    assert_eq!(json["data"]["attributes"]["tokenType"], "Bearer");

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_login_invalid_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let login_request = json!({
        "email": "nonexistent@example.com",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_login_invalid_password() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Create a user
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    app.clone()
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

    // Try to login with wrong password
    let login_request = json!({
        "email": "test@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_login_validation_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Invalid email format
    let login_request = json!({
        "email": "not-an-email",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_refresh_token_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Create a user and login
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    app.clone()
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

    let login_request = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = json["data"]["attributes"]["refreshToken"].as_str().unwrap();

    // Use refresh token
    let refresh_request = json!({
        "refresh_token": refresh_token
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/refresh")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(refresh_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify new tokens are returned
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"]["attributes"]["accessToken"].is_string());
    assert!(json["data"]["attributes"]["refreshToken"].is_string());

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_refresh_token_invalid() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let refresh_request = json!({
        "refresh_token": "invalid.token.here"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/refresh")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(refresh_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_user_extractor_missing_header() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Try to access protected endpoint without auth header
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_user_extractor_invalid_format() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Try with invalid auth header format (not Bearer)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("GET")
                .header("authorization", "Basic sometoken")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_user_extractor_invalid_token() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Try with invalid token
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("GET")
                .header("authorization", "Bearer invalid.token.here")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_user_extractor_refresh_token_rejected() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Create user and login to get tokens
    let create_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    app.clone()
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

    let login_request = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(login_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(login_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = json["data"]["attributes"]["refreshToken"].as_str().unwrap();

    // Try to use refresh token as access token (should be rejected)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users")
                .method("GET")
                .header("authorization", format!("Bearer {}", refresh_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    common::cleanup_test_db(&pool).await;
}
