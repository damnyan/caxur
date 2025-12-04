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

use crate::domain::password::PasswordHashingService;

pub struct LoginUseCase {
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    auth_service: Arc<dyn AuthService>,
    password_service: Arc<dyn PasswordHashingService>,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl LoginUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
        auth_service: Arc<dyn AuthService>,
        password_service: Arc<dyn PasswordHashingService>,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            auth_service,
            password_service,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    #[tracing::instrument(skip(self, req), fields(email = %req.email))]
    pub async fn execute(&self, req: LoginRequest) -> Result<LoginResponse, AppError> {
        // Find user by email
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|e| AppError::InternalServerError(e))?
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Verify password
        let is_valid = self
            .password_service
            .verify_password(&req.password, &user.password_hash)
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
        let password_service = Arc::new(PasswordService::new());

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

        let use_case = LoginUseCase::new(
            user_repo,
            refresh_repo,
            auth_service,
            password_service,
            3600,
            7200,
        );

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
        let password_service = Arc::new(PasswordService::new());

        let use_case = LoginUseCase::new(
            user_repo,
            refresh_repo,
            auth_service,
            password_service,
            3600,
            7200,
        );

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
        let password_service = Arc::new(PasswordService::new());

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

        let use_case = LoginUseCase::new(
            user_repo,
            refresh_repo,
            auth_service,
            password_service,
            3600,
            7200,
        );

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

    struct FailingUserRepository;

    #[async_trait]
    impl UserRepository for FailingUserRepository {
        async fn create(&self, _new_user: NewUser) -> Result<User, anyhow::Error> {
            unimplemented!()
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, anyhow::Error> {
            unimplemented!()
        }
        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, anyhow::Error> {
            Err(anyhow::anyhow!("Database error"))
        }
        async fn find_all(&self, _limit: i64, _offset: i64) -> Result<Vec<User>, anyhow::Error> {
            unimplemented!()
        }
        async fn count(&self) -> Result<i64, anyhow::Error> {
            unimplemented!()
        }
        async fn update(
            &self,
            _id: Uuid,
            _update: crate::domain::users::UpdateUser,
        ) -> Result<User, anyhow::Error> {
            unimplemented!()
        }
        async fn delete(&self, _id: Uuid) -> Result<bool, anyhow::Error> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_login_db_error() {
        let user_repo = Arc::new(FailingUserRepository);
        let refresh_repo = Arc::new(MockRefreshTokenRepository);
        let auth_service = Arc::new(MockAuthService);
        let password_service = Arc::new(PasswordService::new());

        let use_case = LoginUseCase::new(
            user_repo,
            refresh_repo,
            auth_service,
            password_service,
            3600,
            7200,
        );

        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InternalServerError(e) => assert_eq!(e.to_string(), "Database error"),
            _ => panic!("Expected InternalServerError"),
        }
    }

    struct FailingPasswordService;

    #[async_trait]
    impl PasswordHashingService for FailingPasswordService {
        fn hash_password(&self, _password: &str) -> Result<String, anyhow::Error> {
            Err(anyhow::anyhow!("Hashing error"))
        }
        fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool, anyhow::Error> {
            Err(anyhow::anyhow!("Verification error"))
        }
    }

    #[tokio::test]
    async fn test_login_password_verification_error() {
        let user_repo = Arc::new(MockUserRepository::default());
        let refresh_repo = Arc::new(MockRefreshTokenRepository);
        let auth_service = Arc::new(MockAuthService);
        let password_service = Arc::new(FailingPasswordService);

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

        let use_case = LoginUseCase::new(
            user_repo,
            refresh_repo,
            auth_service,
            password_service,
            3600,
            7200,
        );

        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InternalServerError(e) => assert_eq!(e.to_string(), "Verification error"),
            _ => panic!("Expected InternalServerError"),
        }
    }
}
