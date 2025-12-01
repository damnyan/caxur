use crate::application::auth::token_utils::{TokenResponse, generate_and_store_tokens, hash_token};
use crate::domain::auth::{AuthService, RefreshTokenRepository};
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

pub type RefreshTokenResponse = TokenResponse;

pub struct RefreshTokenUseCase {
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl RefreshTokenUseCase {
    pub fn new(
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            refresh_token_repo,
            auth_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    pub async fn execute(
        &self,
        req: RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse, AppError> {
        // Validate the refresh token
        let claims = self
            .auth_service
            .validate_token(&req.refresh_token)
            .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

        // Verify token type
        if claims.token_type != "refresh" {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        // Hash the refresh token to find it in the database
        let token_hash = hash_token(&req.refresh_token);

        // Find refresh token in database
        let stored_token = self
            .refresh_token_repo
            .find_by_hash(&token_hash)
            .await
            .map_err(|e| AppError::InternalServerError(e))?
            .ok_or_else(|| {
                AppError::Unauthorized("Refresh token not found or expired".to_string())
            })?;

        // Parse user ID from claims
        let user_id = claims
            .user_id()
            .map_err(|e| AppError::InternalServerError(e))?;

        // Verify user_id matches
        if stored_token.user_id != user_id {
            return Err(AppError::Unauthorized("Token user mismatch".to_string()));
        }

        // Delete old refresh token (rotation)
        self.refresh_token_repo
            .delete_by_hash(&token_hash)
            .await
            .map_err(|e| AppError::InternalServerError(e))?;

        // Generate and store new token pair (preserve user_type)
        generate_and_store_tokens(
            user_id,
            claims.user_type,
            &self.auth_service,
            &self.refresh_token_repo,
            self.access_token_expiry,
            self.refresh_token_expiry,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::{Claims, NewRefreshToken, RefreshToken};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use time::OffsetDateTime;
    use uuid::Uuid;

    struct MockAuthService {
        validate_result: Option<Result<Claims, anyhow::Error>>,
    }

    impl MockAuthService {
        fn new() -> Self {
            Self {
                validate_result: None,
            }
        }

        fn with_validate_result(mut self, result: Result<Claims, anyhow::Error>) -> Self {
            self.validate_result = Some(result);
            self
        }
    }

    #[async_trait]
    impl AuthService for MockAuthService {
        fn generate_access_token(
            &self,
            _user_id: Uuid,
            _user_type: String,
        ) -> Result<String, anyhow::Error> {
            Ok("access_token".to_string())
        }

        fn generate_refresh_token(
            &self,
            _user_id: Uuid,
            _user_type: String,
        ) -> Result<String, anyhow::Error> {
            Ok("refresh_token".to_string())
        }

        fn validate_token(&self, _token: &str) -> Result<Claims, anyhow::Error> {
            if let Some(res) = &self.validate_result {
                match res {
                    Ok(claims) => Ok(claims.clone()),
                    Err(_) => Err(anyhow::anyhow!("Invalid token")),
                }
            } else {
                Err(anyhow::anyhow!("Mock not configured"))
            }
        }
    }

    struct MockRefreshTokenRepository {
        token: Option<RefreshToken>,
        error: Option<String>,
    }

    impl MockRefreshTokenRepository {
        fn new() -> Self {
            Self {
                token: None,
                error: None,
            }
        }

        fn with_token(mut self, token: RefreshToken) -> Self {
            self.token = Some(token);
            self
        }

        fn with_error(mut self, error: String) -> Self {
            self.error = Some(error);
            self
        }
    }

    #[async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn create(&self, token: NewRefreshToken) -> Result<RefreshToken, anyhow::Error> {
            Ok(RefreshToken {
                id: Uuid::new_v4(),
                user_id: token.user_id,
                user_type: token.user_type,
                token_hash: token.token_hash,
                expires_at: token.expires_at,
                created_at: OffsetDateTime::now_utc(),
            })
        }

        async fn find_by_hash(
            &self,
            _token_hash: &str,
        ) -> Result<Option<RefreshToken>, anyhow::Error> {
            if let Some(err) = &self.error {
                return Err(anyhow::anyhow!(err.clone()));
            }
            Ok(self.token.clone())
        }

        async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64, anyhow::Error> {
            Ok(0)
        }

        async fn delete_expired(&self) -> Result<u64, anyhow::Error> {
            Ok(0)
        }

        async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool, anyhow::Error> {
            if let Some(err) = &self.error {
                return Err(anyhow::anyhow!(err.clone()));
            }
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_refresh_token_success() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_refresh_token(user_id, "user".to_string(), 3600);

        let auth_service = Arc::new(MockAuthService::new().with_validate_result(Ok(claims)));

        let stored_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            user_type: "user".to_string(),
            token_hash: "hash".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
            created_at: OffsetDateTime::now_utc(),
        };

        let refresh_repo = Arc::new(MockRefreshTokenRepository::new().with_token(stored_token));

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "valid_refresh_token".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.access_token, "access_token");
    }

    #[tokio::test]
    async fn test_refresh_token_invalid() {
        let auth_service =
            Arc::new(MockAuthService::new().with_validate_result(Err(anyhow::anyhow!("Invalid"))));
        let refresh_repo = Arc::new(MockRefreshTokenRepository::new());

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "invalid_token".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Invalid refresh token"),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_wrong_type() {
        let user_id = Uuid::new_v4();
        // Create access token instead of refresh token
        let claims = Claims::new_access_token(user_id, "user".to_string(), 3600);

        let auth_service = Arc::new(MockAuthService::new().with_validate_result(Ok(claims)));
        let refresh_repo = Arc::new(MockRefreshTokenRepository::new());

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "access_token_as_refresh".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Invalid token type"),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_not_found() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_refresh_token(user_id, "user".to_string(), 3600);

        let auth_service = Arc::new(MockAuthService::new().with_validate_result(Ok(claims)));
        // Repo returns None by default
        let refresh_repo = Arc::new(MockRefreshTokenRepository::new());

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "valid_token_but_not_in_db".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Refresh token not found or expired"),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_user_mismatch() {
        let user_id = Uuid::new_v4();
        let other_user_id = Uuid::new_v4();
        let claims = Claims::new_refresh_token(user_id, "user".to_string(), 3600);

        let auth_service = Arc::new(MockAuthService::new().with_validate_result(Ok(claims)));

        let stored_token = RefreshToken {
            id: Uuid::new_v4(),
            user_id: other_user_id, // Mismatch
            user_type: "user".to_string(),
            token_hash: "hash".to_string(),
            expires_at: OffsetDateTime::now_utc() + time::Duration::hours(1),
            created_at: OffsetDateTime::now_utc(),
        };

        let refresh_repo = Arc::new(MockRefreshTokenRepository::new().with_token(stored_token));

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "valid_token".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Token user mismatch"),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_refresh_token_repo_error() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new_refresh_token(user_id, "user".to_string(), 3600);

        let auth_service = Arc::new(MockAuthService::new().with_validate_result(Ok(claims)));
        let refresh_repo =
            Arc::new(MockRefreshTokenRepository::new().with_error("DB Error".to_string()));

        let use_case = RefreshTokenUseCase::new(refresh_repo, auth_service, 3600, 7200);

        let req = RefreshTokenRequest {
            refresh_token: "valid_token".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InternalServerError(_) => {}
            _ => panic!("Expected InternalServerError"),
        }
    }
}
