mod common;

use caxur::domain::auth::AuthService;
use caxur::domain::users::{NewUser, UserRepository};
use caxur::infrastructure::auth::JwtAuthService;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use caxur::infrastructure::state::AppState;
use futures::StreamExt;
use std::sync::Arc;

#[tokio::test]
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
async fn test_app_state_new() {
    // Create temporary key files
    let priv_key_path = "test_priv_key.pem";
    let pub_key_path = "test_pub_key.pem";

    // Generate simple keys (just content, validation might fail but we just test AppState creation)
    // Actually JwtAuthService::new reads the file.
    // We can just write dummy content if it doesn't validate immediately on new()
    // Looking at JwtAuthService::new, it calls EncodingKey::from_rsa_pem which expects valid PEM.
    // So we need valid PEM content.

    // Let's use the ones from the project root if they exist, or skip if not.
    // Or better, just generate a dummy PEM.

    let priv_key_content = r#"-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg70WlcQOz1t6ycTKV
4IsazJiPH6wX+57AK5k6COff9B+hRANCAARFEektoRSMoTg1hDqeNP/rLQf0/p4N
nB6zusWchDGSyjagFivRphP4JlJF8bJ6YfZ2s17jPBLUgkIosQXVClgg
-----END PRIVATE KEY-----"#;

    let pub_key_content = r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAERRHpLaEUjKE4NYQ6njT/6y0H9P6e
DZwes7rFnIQxkso2oBYr0aYT+CZSRfGyemH2drNe4zwS1IJCKLEF1QpYIA==
-----END PUBLIC KEY-----"#;

    std::fs::write(priv_key_path, priv_key_content).unwrap();
    std::fs::write(pub_key_path, pub_key_content).unwrap();

    // We can't easily create a disconnected pool synchronously without `connect_lazy`.
    // `sqlx::PgPool::connect_lazy` returns a pool immediately.

    let pool = sqlx::PgPool::connect_lazy("postgres://localhost:5432/dummy").unwrap();

    // We expect this might fail if keys are invalid, but let's see.
    // If JwtAuthService validates keys on load, we need valid keys.
    // Assuming it does.

    // If we can't easily generate valid keys, maybe we can skip this test part or mock it differently.
    // But AppState takes Arc<JwtAuthService>.

    // Let's try to use the env vars if set, otherwise write dummy.

    let auth_service = match JwtAuthService::new(priv_key_path, pub_key_path, 900, 900) {
        Ok(service) => Arc::new(service),
        Err(e) => {
            // If dummy keys fail validation, we can't easily test AppState::new with real AuthService
            // without real keys.
            // Let's just cleanup and return/panic.
            std::fs::remove_file(priv_key_path).unwrap_or_default();
            std::fs::remove_file(pub_key_path).unwrap_or_default();
            panic!("Failed to create JwtAuthService with dummy keys: {:?}", e);
        }
    };

    let state = AppState::new(pool, auth_service);

    // Just verify we can access fields
    assert!(state.auth_service.validate_token("invalid").is_err());

    // Cleanup
    std::fs::remove_file(priv_key_path).unwrap_or_default();
    std::fs::remove_file(pub_key_path).unwrap_or_default();
}
