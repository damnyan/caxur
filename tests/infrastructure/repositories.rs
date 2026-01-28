use crate::common;

use caxur::domain::administrators::AdministratorRepository;
use caxur::domain::auth::AuthService;
use caxur::domain::users::{NewUser, UserRepository};
use caxur::infrastructure::auth::JwtAuthService;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use caxur::infrastructure::state::AppState;
use futures::StreamExt;
use serial_test::serial;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_postgres_user_repo_batch_create() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = PostgresUserRepository::new(pool.clone());

    let new_users = vec![
        NewUser {
            username: "batch1".to_string(),
            email: "batch1@example.com".to_string(),
            password_hash: "hash1".to_string(),
        },
        NewUser {
            username: "batch2".to_string(),
            email: "batch2@example.com".to_string(),
            password_hash: "hash2".to_string(),
        },
    ];

    let created_users = repo
        .batch_create(new_users)
        .await
        .expect("Failed to batch create users");

    assert_eq!(created_users.len(), 2);
    assert_eq!(created_users[0].username, "batch1");
    assert_eq!(created_users[1].username, "batch2");

    let count = repo.count().await.expect("Failed to count users");
    assert_eq!(count, 2);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_postgres_user_repo_find_all_stream() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = PostgresUserRepository::new(pool.clone());

    // Create 5 users
    for i in 0..5 {
        repo.create(NewUser {
            username: format!("stream{}", i),
            email: format!("stream{}@example.com", i),
            password_hash: "hash".to_string(),
        })
        .await
        .expect("Failed to create user");
    }

    // Stream users
    let mut stream = repo.find_all_stream(10, 0);
    let mut count = 0;

    while let Some(result) = stream.next().await {
        assert!(result.is_ok());
        count += 1;
    }

    assert_eq!(count, 5);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_app_state_new() {
    // Use valid key files from fixtures
    let priv_key_path = "tests/fixtures/keys/test_priv_key.pem";
    let pub_key_path = "tests/fixtures/keys/test_pub_key.pem";

    // We can't easily create a disconnected pool synchronously without `connect_lazy`.
    // `sqlx::PgPool::connect_lazy` returns a pool immediately.

    let pool = sqlx::PgPool::connect_lazy("postgres://localhost:5432/dummy").unwrap();

    let auth_service = match JwtAuthService::new(priv_key_path, pub_key_path, 900, 900) {
        Ok(service) => Arc::new(service),
        Err(e) => {
            panic!("Failed to create JwtAuthService with dummy keys: {:?}", e);
        }
    };

    let state = AppState::new(pool, auth_service);

    // Just verify we can access fields
    assert!(state.auth_service.validate_token("invalid").is_err());
}

#[tokio::test]
#[serial]
async fn test_admin_repo_empty_update_non_existent() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo =
        caxur::infrastructure::repositories::administrators::PostgresAdministratorRepository::new(
            pool.clone(),
        );
    let non_existent_id = uuid::Uuid::new_v4();
    let update = caxur::domain::administrators::UpdateAdministrator {
        first_name: None,
        middle_name: None,
        last_name: None,
        suffix: None,
        contact_number: None,
        email: None,
        password_hash: None,
    };

    // This specific case: empty update struct + non-existent ID
    // should trigger the `ok_or_else(|| anyhow::anyhow!("Administrator not found"))`
    // at line 153 of src/infrastructure/repositories/administrators.rs
    let result = repo.update(non_existent_id, update).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Administrator not found");
}

#[tokio::test]
#[serial]
async fn test_app_state_from_ref() {
    let pool = setup_test_db_or_skip!();
    let state = common::create_test_app_state(pool);

    // Explicitly test the FromRef implementation for Arc<JwtAuthService>
    // This covers lines 24-28 in src/infrastructure/state.rs
    let auth_service: Arc<JwtAuthService> = axum::extract::FromRef::from_ref(&state);

    // Check if the pointers are the same, implying it's the exact same Arc
    assert!(Arc::ptr_eq(&state.auth_service, &auth_service));
}
