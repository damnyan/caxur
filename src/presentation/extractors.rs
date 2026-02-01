use crate::domain::auth::{AuthService, Claims};
use crate::infrastructure::state::AppState;
use crate::shared::error::AppError;
use axum::{extract::FromRequestParts, http::request::Parts};

/// Authenticated user extractor
/// Validates JWT token from Authorization header
pub struct AuthUser {
    pub claims: Claims,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
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

        // Use injected auth service
        let auth_service = &state.auth_service;

        // Validate token
        let claims = auth_service
            .validate_token(token)
            .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

        // Verify token type is access token
        if claims.token_type != "access" {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        Ok(AuthUser { claims })
    }
}
