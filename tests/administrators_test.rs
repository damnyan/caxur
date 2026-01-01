mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_create_admin() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    // Generate a valid token for authentication
    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();

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
#[serial]
async fn test_create_admin_duplicate_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    // Create first admin
    let response1 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
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
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_get_admin() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create an admin
    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_id = json["data"]["id"].as_str().unwrap();

    // Get the admin
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", admin_id))
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
#[serial]
async fn test_list_admins() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create an admin
    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List admins
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
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
#[serial]
async fn test_update_admin() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create an admin
    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_id = json["data"]["id"].as_str().unwrap();

    // Update the admin
    let update_request = json!({
        "firstName": "UpdatedAdmin",
        "email": "updated@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", admin_id))
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
#[serial]
async fn test_delete_admin() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create an admin
    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_id = json["data"]["id"].as_str().unwrap();

    // Delete the admin
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", admin_id))
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
#[serial]
async fn test_update_admin_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);
    let fake_id = Uuid::new_v4();

    let update_request = json!({
        "firstName": "UpdatedAdmin"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", fake_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_admin_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);
    let fake_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", fake_id))
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

#[tokio::test]
#[serial]
async fn test_update_admin_duplicate_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create first admin
    let create_request1 = json!({
        "firstName": "Admin1",
        "lastName": "User",
        "email": "admin1@example.com",
        "password": "password123"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Create second admin
    let create_request2 = json!({
        "firstName": "Admin2",
        "lastName": "User",
        "email": "admin2@example.com",
        "password": "password123"
    });

    let create_response2 = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response2.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin2_id = json["data"]["id"].as_str().unwrap();

    // Update second admin with first admin's email
    let update_request = json!({
        "email": "admin1@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", admin2_id))
                .method("PUT")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(update_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_update_admin_password() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);

    // Create admin
    let create_request = json!({
        "firstName": "Admin",
        "lastName": "User",
        "email": "admin@example.com",
        "password": "password123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/administrators")
                .method("POST")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_id = json["data"]["id"].as_str().unwrap();

    // Update password
    let update_request = json!({
        "password": "newpassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", admin_id))
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
#[serial]
async fn test_get_admin_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state);

    let user_id = Uuid::new_v4();
    let token = common::generate_test_token(user_id);
    let fake_id = Uuid::new_v4();

    // Try to get a non-existent admin
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/v1/admin/administrators/{}", fake_id))
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
