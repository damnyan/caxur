use crate::application::auth::login::{LoginRequest, LoginResponse, LoginUseCase};
use crate::application::auth::refresh::{RefreshTokenRequest, RefreshTokenResponse, RefreshTokenUseCase};
use crate::domain::auth::{AuthService, Claims};
use crate::infrastructure::auth::JwtAuthService;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::ApiResponse;
use crate::shared::validation::ValidatedJson;
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Login handler
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Get configuration from environment
    let private_key_path =
        std::env::var("JWT_PRIVATE_KEY_PATH").unwrap_or_else(|_| "keys/private_key.pem".to_string());
    let public_key_path =
        std::env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "keys/public_key.pem".to_string());
    let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "900".to_string())
        .parse::<i64>()
        .unwrap_or(900);
    let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "604800".to_string())
        .parse::<i64>()
        .unwrap_or(604800);

    // Initialize services
    let auth_service = Arc::new(
        JwtAuthService::new(
            &private_key_path,
            &public_key_path,
            access_token_expiry,
            refresh_token_expiry,
        )
        .map_err(|e| AppError::InternalServerError(e))?,
    ) as Arc<dyn AuthService>;

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool));

    // Execute use case
    let use_case = LoginUseCase::new(
        user_repo,
        refresh_token_repo,
        auth_service,
        access_token_expiry,
        refresh_token_expiry,
    );

    let response = use_case.execute(req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(response))))
}

/// Refresh token handler
#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<RefreshTokenResponse>),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Get configuration from environment
    let private_key_path =
        std::env::var("JWT_PRIVATE_KEY_PATH").unwrap_or_else(|_| "keys/private_key.pem".to_string());
    let public_key_path =
        std::env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "keys/public_key.pem".to_string());
    let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "900".to_string())
        .parse::<i64>()
        .unwrap_or(900);
    let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "604800".to_string())
        .parse::<i64>()
        .unwrap_or(604800);

    // Initialize services
    let auth_service = Arc::new(
        JwtAuthService::new(
            &private_key_path,
            &public_key_path,
            access_token_expiry,
            refresh_token_expiry,
        )
        .map_err(|e| AppError::InternalServerError(e))?,
    ) as Arc<dyn AuthService>;

    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool));

    // Execute use case
    let use_case = RefreshTokenUseCase::new(
        refresh_token_repo,
        auth_service,
        access_token_expiry,
        refresh_token_expiry,
    );

    let response = use_case.execute(req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(response))))
}

/// Authenticated user extractor
/// Validates JWT token from Authorization header
pub struct AuthUser {
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        // Validate Bearer scheme
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        // Extract token
        let token = &auth_header[7..];

        // Get configuration from environment
        let private_key_path =
            std::env::var("JWT_PRIVATE_KEY_PATH").unwrap_or_else(|_| "keys/private_key.pem".to_string());
        let public_key_path =
            std::env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "keys/public_key.pem".to_string());
        let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "900".to_string())
            .parse::<i64>()
            .unwrap_or(900);
        let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
            .unwrap_or_else(|_| "604800".to_string())
            .parse::<i64>()
            .unwrap_or(604800);

        // Initialize auth service
        let auth_service = JwtAuthService::new(
            &private_key_path,
            &public_key_path,
            access_token_expiry,
            refresh_token_expiry,
        )
        .map_err(|e| AppError::InternalServerError(e))?;

        // Validate token
        let claims = auth_service
            .validate_token(token)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        // Verify token type is access token
        if claims.token_type != "access" {
            return Err(AppError::Unauthorized(
                "Invalid token type".to_string(),
            ));
        }

        Ok(AuthUser { claims })
    }
}
