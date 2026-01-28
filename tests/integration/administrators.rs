use crate::common;

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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
    let app = caxur::presentation::router::app(state).unwrap();

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

// =========================================================================
// Use Case Unit/Component Tests (Moved from tests/application/administrators_create_test.rs)
// =========================================================================

use caxur::application::administrators::create::{
    CreateAdministratorRequest, CreateAdministratorUseCase,
};
use caxur::domain::administrators::{
    Administrator, AdministratorRepository, NewAdministrator, UpdateAdministrator,
};
use caxur::infrastructure::password::PasswordService;
use caxur::infrastructure::repositories::administrators::PostgresAdministratorRepository;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_use_case_create_admin_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateAdministratorUseCase::new(repo, password_service);

    let prefix = Uuid::new_v4().to_string();
    let req = CreateAdministratorRequest {
        first_name: "John".to_string(),
        middle_name: Some("Quincy".to_string()),
        last_name: "Doe".to_string(),
        suffix: None,
        contact_number: Some("1234567890".to_string()),
        email: format!("admin_{}@example.com", prefix),
        password: "password123".to_string(),
    };

    let admin = use_case.execute(req).await.expect("Failed to create admin");

    assert_eq!(admin.first_name, "John");
    assert_eq!(admin.last_name, "Doe");
    assert!(admin.email.contains(&prefix));
    assert_ne!(admin.password_hash, "password123"); // Should be hashed
}

#[tokio::test]
#[serial]
async fn test_use_case_create_admin_duplicate_email() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateAdministratorUseCase::new(repo, password_service);

    let prefix = Uuid::new_v4().to_string();
    let email = format!("dup_admin_{}@example.com", prefix);

    // Create first admin
    let req1 = CreateAdministratorRequest {
        first_name: "Admin1".to_string(),
        middle_name: None,
        last_name: "User".to_string(),
        suffix: None,
        contact_number: None,
        email: email.clone(),
        password: "password123".to_string(),
    };
    use_case
        .execute(req1)
        .await
        .expect("Failed to create first admin");

    // Create second admin with same email
    let req2 = CreateAdministratorRequest {
        first_name: "Admin2".to_string(),
        middle_name: None,
        last_name: "User".to_string(),
        suffix: None,
        contact_number: None,
        email: email,
        password: "password456".to_string(),
    };
    let result = use_case.execute(req2).await;

    assert!(result.is_err());

    match result {
        Err(caxur::shared::error::AppError::ValidationError(msg)) => {
            assert_eq!(msg, "Email already registered");
        }
        _ => panic!("Expected ValidationError, got {:?}", result),
    }
}

use anyhow::anyhow;
use caxur::domain::password::PasswordHashingService;

struct FaultyPasswordHashingService;

#[async_trait::async_trait]
impl PasswordHashingService for FaultyPasswordHashingService {
    fn hash_password(&self, _password: &str) -> anyhow::Result<String> {
        Err(anyhow!("Hashing failed"))
    }

    fn verify_password(&self, _password: &str, _hash: &str) -> anyhow::Result<bool> {
        unimplemented!()
    }
}

#[tokio::test]
async fn test_use_case_create_admin_hash_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let password_service = Arc::new(FaultyPasswordHashingService);
    let use_case = CreateAdministratorUseCase::new(repo, password_service);

    let req = CreateAdministratorRequest {
        first_name: "Hash".to_string(),
        middle_name: None,
        last_name: "Fail".to_string(),
        suffix: None,
        contact_number: None,
        email: "hash_fail@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;

    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Hashing failed");
        }
        _ => panic!(
            "Expected InternalServerError(Hashing failed), got {:?}",
            result
        ),
    }
}

#[tokio::test]
#[serial]
async fn test_use_case_validate_unique_email_direct() {
    // This tests the `validate_unique_email` method on request struct directly
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()))
        as Arc<dyn AdministratorRepository>;

    let prefix = Uuid::new_v4().to_string();
    let email = format!("direct_validate_{}@example.com", prefix);

    // 1. Validate non-existent email (should pass)
    let req = CreateAdministratorRequest {
        first_name: "Test".to_string(),
        middle_name: None,
        last_name: "User".to_string(),
        suffix: None,
        contact_number: None,
        email: email.clone(),
        password: "password123".to_string(),
    };

    assert!(req.validate_unique_email(&repo).await.is_ok());

    // 2. Create the user
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateAdministratorUseCase::new(repo.clone(), password_service);
    use_case.execute(req).await.expect("Failed to create user");

    // 3. Validate existing email (should fail)
    let req2 = CreateAdministratorRequest {
        first_name: "Test".to_string(),
        middle_name: None,
        last_name: "User".to_string(),
        suffix: None,
        contact_number: None,
        email: email.clone(),
        password: "password123".to_string(),
    };

    match req2.validate_unique_email(&repo).await {
        Err(caxur::shared::error::AppError::ValidationError(msg)) => {
            assert_eq!(msg, "Email already exists");
        }
        _ => panic!("Expected ValidationError"),
    }
}

struct FaultyAdministratorRepository;

#[async_trait::async_trait]
impl AdministratorRepository for FaultyAdministratorRepository {
    async fn create(&self, _new_admin: NewAdministrator) -> anyhow::Result<Administrator> {
        Err(anyhow::anyhow!("Database connection failed"))
    }

    async fn find_by_email(&self, _email: &str) -> anyhow::Result<Option<Administrator>> {
        // Must return Ok(None) to pass the initial validation check
        Ok(None)
    }

    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<Administrator>> {
        unimplemented!()
    }
    async fn find_all(&self, _limit: i64, _offset: i64) -> anyhow::Result<Vec<Administrator>> {
        unimplemented!()
    }
    async fn count(&self) -> anyhow::Result<i64> {
        unimplemented!()
    }
    async fn update(
        &self,
        _id: Uuid,
        _update: UpdateAdministrator,
    ) -> anyhow::Result<Administrator> {
        unimplemented!()
    }
    async fn delete(&self, _id: Uuid) -> anyhow::Result<bool> {
        unimplemented!()
    }
}

#[tokio::test]
async fn test_use_case_create_admin_repo_error() {
    let repo = Arc::new(FaultyAdministratorRepository);
    let password_service = Arc::new(PasswordService::new());
    let use_case = CreateAdministratorUseCase::new(repo, password_service);

    let req = CreateAdministratorRequest {
        first_name: "Repo".to_string(),
        middle_name: None,
        last_name: "Fail".to_string(),
        suffix: None,
        contact_number: None,
        email: "repo_fail@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = use_case.execute(req).await;

    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Database connection failed");
        }
        _ => panic!(
            "Expected InternalServerError(Database connection failed), got {:?}",
            result
        ),
    }
}
