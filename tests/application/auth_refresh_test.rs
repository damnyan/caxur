use crate::common;
use crate::setup_test_db_or_skip;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use caxur::application::auth::refresh::{RefreshTokenRequest, RefreshTokenUseCase};
use caxur::domain::auth::{
    AuthService, Claims, NewRefreshToken, RefreshToken, RefreshTokenRepository,
};
use caxur::domain::users::NewUser;
use caxur::domain::users::UserRepository;
use caxur::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use caxur::infrastructure::repositories::users::PostgresUserRepository;
use serial_test::serial;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

// Mocks for Error Scenarios

struct FindErrorRepository;
#[async_trait]
impl RefreshTokenRepository for FindErrorRepository {
    async fn create(&self, _token: NewRefreshToken) -> Result<RefreshToken> {
        unimplemented!()
    }
    async fn find_by_hash(&self, _token_hash: &str) -> Result<Option<RefreshToken>> {
        Err(anyhow!("Database failure on find"))
    }
    async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_expired(&self) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool> {
        unimplemented!()
    }
}

struct DeleteErrorRepository {
    // We need to return a token on find, then fail on delete
    user_id: Uuid,
    token_hash: String,
}

#[async_trait]
impl RefreshTokenRepository for DeleteErrorRepository {
    async fn create(&self, _token: NewRefreshToken) -> Result<RefreshToken> {
        unimplemented!()
    }
    async fn find_by_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>> {
        if token_hash == self.token_hash {
            Ok(Some(RefreshToken {
                id: Uuid::new_v4(),
                user_id: self.user_id,
                user_type: "user".to_string(),
                token_hash: self.token_hash.clone(),
                expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
                created_at: OffsetDateTime::now_utc(),
            }))
        } else {
            Ok(None)
        }
    }
    async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_expired(&self) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool> {
        Err(anyhow!("Database failure on delete"))
    }
}

struct InvalidSubjectAuthService;
impl AuthService for InvalidSubjectAuthService {
    fn generate_access_token(&self, _user_id: Uuid, _user_type: String) -> Result<String> {
        unimplemented!()
    }
    fn generate_refresh_token(&self, _user_id: Uuid, _user_type: String) -> Result<String> {
        unimplemented!()
    }
    fn validate_token(&self, _token: &str) -> Result<Claims> {
        Ok(Claims {
            sub: "not-a-uuid".to_string(), // <--- This will cause user_id parsing failure
            exp: (OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp(),
            iat: OffsetDateTime::now_utc().unix_timestamp(),
            token_type: "refresh".to_string(),
            user_type: "user".to_string(),
        })
    }
}

#[tokio::test]
async fn test_refresh_repo_find_error() {
    let repo = Arc::new(FindErrorRepository);
    let auth_service = common::create_test_auth_service();

    let token = auth_service
        .generate_refresh_token(Uuid::new_v4(), "user".to_string())
        .unwrap();

    let use_case = RefreshTokenUseCase::new(repo, auth_service, 3600, 7200);

    let req = RefreshTokenRequest {
        refresh_token: token,
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Database failure on find");
        }
        _ => panic!("Expected InternalServerError, got {:?}", result),
    }
}

#[tokio::test]
async fn test_refresh_repo_delete_error() {
    let auth_service = common::create_test_auth_service();
    let token = auth_service
        .generate_refresh_token(Uuid::new_v4(), "user".to_string())
        .unwrap();

    // We need to match the hash logic. The real use case hashes the token from request.
    // The use case calls `hash_token(&req.refresh_token)`.
    // In our mock, we just exact match what we expect.
    // But `hash_token` is internal/utils. We can't easily predict the hash unless we use `hash_token` ourselves.
    // Or we rely on `FindErrorRepository` being simple.
    // For `DeleteErrorRepository`, we need to know the hash.
    // Let's rely on the fact that `hash_token` is deterministic and sha256.
    // We can call the public utility if available, or just reproduce it.
    // `caxur::application::auth::token_utils::hash_token` is public?
    // refresh.rs imports it.

    let hash = caxur::application::auth::token_utils::hash_token(&token);
    // Wait, `hash_token` might not be exposed from `caxur::application::auth::token_utils` to tests/lib
    // unless `token_utils` is pub. It is.

    let _repo = Arc::new(DeleteErrorRepository {
        user_id: Uuid::nil(), // Doesn't matter for this test as long as claims match?
        // Wait, validate_token will return claims with a REAL user_id from the token generated above.
        // We need the repo to return a token with that SAME user_id to pass the check on line 72.
        token_hash: hash,
    });

    // We need to know the user_id inside the token
    let claims = auth_service.validate_token(&token).unwrap();
    let user_id = claims.user_id().unwrap();

    // Re-create repo with correct user_id
    let repo = Arc::new(DeleteErrorRepository {
        user_id,
        token_hash: caxur::application::auth::token_utils::hash_token(&token),
    });

    let use_case = RefreshTokenUseCase::new(repo, auth_service, 3600, 7200);

    let req = RefreshTokenRequest {
        refresh_token: token,
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::InternalServerError(e)) => {
            assert_eq!(e.to_string(), "Database failure on delete");
        }
        _ => panic!("Expected InternalServerError, got {:?}", result),
    }
}

#[tokio::test]
async fn test_refresh_user_id_parse_error() {
    let _repo = Arc::new(FindErrorRepository); // Won't be reached or doesn't matter?
    // Wait, execute() order:
    // 1. validate_token -> Claims
    // 2. hash_token
    // 3. find_by_hash
    // 4. claims.user_id() check

    // The `claims.user_id()` call (line 68) happens AFTER `find_by_hash`.
    // So we need `find_by_hash` to SUCCEED.
    // `FindErrorRepository` fails find. So we need a success mock for find.

    let user_id = Uuid::new_v4();
    let _token_hash = "some_hash".to_string();

    // We can use DeleteErrorRepository but make it succeed on delete too? Or simpler mock.
    struct SuccessRepository {
        user_id: Uuid,
    }
    #[async_trait]
    impl RefreshTokenRepository for SuccessRepository {
        async fn create(&self, _token: NewRefreshToken) -> Result<RefreshToken> {
            unimplemented!()
        }
        async fn find_by_hash(&self, _token_hash: &str) -> Result<Option<RefreshToken>> {
            Ok(Some(RefreshToken {
                id: Uuid::new_v4(),
                user_id: self.user_id,
                user_type: "user".to_string(),
                token_hash: "irrelevant".to_string(),
                expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
                created_at: OffsetDateTime::now_utc(),
            }))
        }
        async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64> {
            unimplemented!()
        }
        async fn delete_expired(&self) -> Result<u64> {
            unimplemented!()
        }
        async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool> {
            Ok(true)
        }
    }

    let repo = Arc::new(SuccessRepository { user_id });
    let auth_service = Arc::new(InvalidSubjectAuthService); // Returns "not-a-uuid"

    let use_case = RefreshTokenUseCase::new(repo, auth_service, 3600, 7200);

    let req = RefreshTokenRequest {
        refresh_token: "ignored".to_string(),
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::InternalServerError(_)) => {
            // Success catching parser error
        }
        err => panic!("Expected InternalServerError (parse), got {:?}", err),
    }
}

#[tokio::test]
#[serial]
async fn test_refresh_token_mismatch() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let auth_service = common::create_test_auth_service();

    // Generate a real token for User A
    let user_a_id = Uuid::new_v4();
    let token_a = auth_service
        .generate_refresh_token(user_a_id, "user".to_string())
        .unwrap();
    let hash_a = caxur::application::auth::token_utils::hash_token(&token_a);

    // Manually insert a token record with the SAME HASH but DIFFERENT User ID (User B)
    // This simulates a collision or data corruption or attack where hash matches but user doesn't.
    // Wait, hash collision is unlikely.
    // More likely logic error: we found a token by hash, but the token claims says user A, and DB says user B.
    // How can this happen?
    // If I use a valid token for User A, but I insert it into DB associated with User B.

    // We need to insert directly avoiding `hash_token` if we want to fake it?
    // Or just use `create` with `hash_a` but `user_b_id`.

    // Create User B first (FK constraint)
    let user_repo = PostgresUserRepository::new(pool.clone());
    user_repo
        .create(NewUser {
            username: "user_b".to_string(),
            email: "user_b@example.com".to_string(),
            password_hash: "hash".to_string(),
        })
        .await
        .unwrap();
    // We need the ID of user B.
    let user_b = user_repo
        .find_by_email("user_b@example.com")
        .await
        .unwrap()
        .unwrap();

    let new_token = NewRefreshToken {
        user_id: user_b.id, // Associated with User B in DB
        user_type: "user".to_string(),
        token_hash: hash_a, // But hash matches Token A
        expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
    };
    repo.create(new_token).await.unwrap();

    let use_case = RefreshTokenUseCase::new(repo, auth_service, 3600, 7200);

    let req = RefreshTokenRequest {
        refresh_token: token_a, // Claims say User A
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::Unauthorized(msg)) => {
            assert_eq!(msg, "Token user mismatch");
        }
        _ => panic!(
            "Expected Unauthorized(Token user mismatch), got {:?}",
            result
        ),
    }
}

struct NotFoundRepository;
#[async_trait]
impl RefreshTokenRepository for NotFoundRepository {
    async fn create(&self, _token: NewRefreshToken) -> Result<RefreshToken> {
        unimplemented!()
    }
    async fn find_by_hash(&self, _token_hash: &str) -> Result<Option<RefreshToken>> {
        Ok(None)
    }
    async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_expired(&self) -> Result<u64> {
        unimplemented!()
    }
    async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool> {
        unimplemented!()
    }
}

#[tokio::test]
async fn test_refresh_token_not_found() {
    let repo = Arc::new(NotFoundRepository);
    let auth_service = common::create_test_auth_service();

    let use_case = RefreshTokenUseCase::new(repo, auth_service.clone(), 3600, 7200);

    // We need a valid token string so it passes validation,
    // but the repo will say it's not found.
    let token = auth_service
        .generate_refresh_token(Uuid::new_v4(), "user".to_string())
        .unwrap();

    let req = RefreshTokenRequest {
        refresh_token: token,
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::Unauthorized(msg)) => {
            assert_eq!(msg, "Refresh token not found or expired");
        }
        _ => panic!(
            "Expected Unauthorized(Refresh token not found or expired), got {:?}",
            result
        ),
    }
}

struct WrongTypeAuthService;
impl AuthService for WrongTypeAuthService {
    fn generate_access_token(&self, _user_id: Uuid, _user_type: String) -> Result<String> {
        unimplemented!()
    }
    fn generate_refresh_token(&self, _user_id: Uuid, _user_type: String) -> Result<String> {
        unimplemented!()
    }
    fn validate_token(&self, _token: &str) -> Result<Claims> {
        Ok(Claims {
            sub: Uuid::new_v4().to_string(),
            exp: (OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp(),
            iat: OffsetDateTime::now_utc().unix_timestamp(),
            token_type: "access".to_string(), // Invalid type for refresh
            user_type: "user".to_string(),
        })
    }
}

#[tokio::test]
async fn test_refresh_token_type_mismatch() {
    let repo = Arc::new(NotFoundRepository); // Won't be reached or finds nothing, doesn't matter
    let auth_service = Arc::new(WrongTypeAuthService);

    let use_case = RefreshTokenUseCase::new(repo, auth_service, 3600, 7200);

    let req = RefreshTokenRequest {
        refresh_token: "ignored".to_string(),
    };

    let result = use_case.execute(req).await;
    match result {
        Err(caxur::shared::error::AppError::Unauthorized(msg)) => {
            assert_eq!(msg, "Invalid token type");
        }
        _ => panic!(
            "Expected Unauthorized(Invalid token type), got {:?}",
            result
        ),
    }
}
