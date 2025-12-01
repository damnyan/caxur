use crate::application::auth::token_utils::{TokenResponse, generate_and_store_tokens};
use crate::domain::auth::{AuthService, RefreshTokenRepository};
use crate::domain::password::PasswordService;
use crate::domain::users::UserRepository;
use crate::shared::error::AppError;
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

pub type LoginResponse = TokenResponse;

pub struct LoginUseCase {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl LoginUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            auth_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    pub async fn execute(&self, req: LoginRequest) -> Result<LoginResponse, AppError> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalServerError(e))?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Verify password
        let is_valid = PasswordService::verify_password(&req.password, &user.password_hash)
            .map_err(|e| AppError::InternalServerError(e))?;

        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        // Generate and store token pair
        generate_and_store_tokens(
            user.id,
            "user".to_string(),
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
    use crate::domain::users::{NewUser, User};
    use crate::infrastructure::repositories::mock::MockUserRepository;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;
    use uuid::Uuid;

    struct MockAuthService;

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
            unimplemented!()
        }
    }

    struct MockRefreshTokenRepository;

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
            unimplemented!()
        }

        async fn delete_by_user_id(&self, _user_id: Uuid) -> Result<u64, anyhow::Error> {
            Ok(0)
        }

        async fn delete_expired(&self) -> Result<u64, anyhow::Error> {
            Ok(0)
        }

        async fn delete_by_hash(&self, _token_hash: &str) -> Result<bool, anyhow::Error> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        let user_repo = Arc::new(MockUserRepository::default());
        let refresh_repo = Arc::new(MockRefreshTokenRepository);
        let auth_service = Arc::new(MockAuthService);

        // Create user
        let password = "password123";
        let hash = PasswordService::hash_password(password).unwrap();
        let user = user_repo
            .create(NewUser {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                password_hash: hash,
            })
            .await
            .unwrap();

        let use_case = LoginUseCase::new(user_repo, refresh_repo, auth_service, 3600, 7200);

        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.access_token, "access_token");
    }

    #[tokio::test]
    async fn test_login_invalid_email() {
        let user_repo = Arc::new(MockUserRepository::default());
        let refresh_repo = Arc::new(MockRefreshTokenRepository);
        let auth_service = Arc::new(MockAuthService);

        let use_case = LoginUseCase::new(user_repo, refresh_repo, auth_service, 3600, 7200);

        let req = LoginRequest {
            email: "wrong@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Invalid email or password"),
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let user_repo = Arc::new(MockUserRepository::default());
        let refresh_repo = Arc::new(MockRefreshTokenRepository);
        let auth_service = Arc::new(MockAuthService);

        // Create user
        let password = "password123";
        let hash = PasswordService::hash_password(password).unwrap();
        let _user = user_repo
            .create(NewUser {
                username: "testuser".to_string(),
                email: "test@example.com".to_string(),
                password_hash: hash,
            })
            .await
            .unwrap();

        let use_case = LoginUseCase::new(user_repo, refresh_repo, auth_service, 3600, 7200);

        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Invalid email or password"),
            _ => panic!("Expected Unauthorized error"),
        }
    }
}
