mod common;

use caxur::domain::auth::{NewRefreshToken, RefreshTokenRepository};
use caxur::domain::users::{NewUser, UserRepository};
use caxur::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use serial_test::serial;
use time::OffsetDateTime;
use uuid::Uuid;

/// Helper function to create a test user
async fn create_test_user(pool: &sqlx::PgPool) -> Uuid {
    let repo = PostgresUserRepository::new(pool.clone());
    let new_user = NewUser {
        username: format!("testuser_{}", Uuid::new_v4()),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = repo.create(new_user).await.unwrap();
    user.id
}

#[tokio::test]
#[serial]
async fn test_create_refresh_token() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    let new_token = NewRefreshToken {
        user_id,
        user_type: "user".to_string(),
        token_hash: "test_hash_123".to_string(),
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
    };

    let result = repo.create(new_token).await;
    assert!(result.is_ok());

    let token = result.unwrap();
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.user_type, "user");
    assert_eq!(token.token_hash, "test_hash_123");

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_find_by_hash_existing() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    // Create a token
    let new_token = NewRefreshToken {
        user_id,
        user_type: "user".to_string(),
        token_hash: "find_me_hash".to_string(),
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
    };

    repo.create(new_token).await.unwrap();

    // Find it by hash
    let result = repo.find_by_hash("find_me_hash").await;
    assert!(result.is_ok());

    let token = result.unwrap();
    assert!(token.is_some());

    let token = token.unwrap();
    assert_eq!(token.user_id, user_id);
    assert_eq!(token.token_hash, "find_me_hash");

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_find_by_hash_nonexistent() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    let result = repo.find_by_hash("nonexistent_hash").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_find_by_hash_expired() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    // Create an expired token
    let new_token = NewRefreshToken {
        user_id,
        user_type: "user".to_string(),
        token_hash: "expired_hash".to_string(),
        expires_at: OffsetDateTime::now_utc() - time::Duration::days(1), // Expired yesterday
    };

    repo.create(new_token).await.unwrap();

    // Try to find it - should return None because it's expired
    let result = repo.find_by_hash("expired_hash").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_by_user_id() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    // Create multiple tokens for the same user
    for i in 0..3 {
        let new_token = NewRefreshToken {
            user_id,
            user_type: "user".to_string(),
            token_hash: format!("hash_{}", i),
            expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
        };
        repo.create(new_token).await.unwrap();
    }

    // Delete all tokens for this user
    let result = repo.delete_by_user_id(user_id).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    // Verify they're deleted
    for i in 0..3 {
        let token = repo.find_by_hash(&format!("hash_{}", i)).await.unwrap();
        assert!(token.is_none());
    }

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_by_hash() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    let new_token = NewRefreshToken {
        user_id,
        user_type: "user".to_string(),
        token_hash: "delete_me".to_string(),
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
    };

    repo.create(new_token).await.unwrap();

    // Delete by hash
    let result = repo.delete_by_hash("delete_me").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    // Verify it's deleted
    let token = repo.find_by_hash("delete_me").await.unwrap();
    assert!(token.is_none());

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_by_hash_nonexistent() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    let result = repo.delete_by_hash("nonexistent").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_delete_expired() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = create_test_user(&pool).await;
    let repo = PostgresRefreshTokenRepository::new(pool.clone());

    // Create 2 expired tokens
    for i in 0..2 {
        let new_token = NewRefreshToken {
            user_id,
            user_type: "user".to_string(),
            token_hash: format!("expired_{}", i),
            expires_at: OffsetDateTime::now_utc() - time::Duration::days(1),
        };
        repo.create(new_token).await.unwrap();
    }

    // Create 1 valid token
    let new_token = NewRefreshToken {
        user_id,
        user_type: "user".to_string(),
        token_hash: "valid_token".to_string(),
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(7),
    };
    repo.create(new_token).await.unwrap();

    // Delete expired tokens
    let result = repo.delete_expired().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);

    // Verify valid token still exists
    let token = repo.find_by_hash("valid_token").await.unwrap();
    assert!(token.is_some());

    common::cleanup_test_db(&pool).await;
}
