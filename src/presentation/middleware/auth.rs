use crate::domain::administrators::AdministratorRepository;
use crate::domain::permissions::Permission;
use crate::infrastructure::repositories::administrators::PostgresAdministratorRepository;
use crate::infrastructure::state::AppState; // Correct import
use crate::presentation::extractors::AuthUser;
use crate::shared::error::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing;

#[derive(Clone)]
pub struct RequiredPermissions {
    pub user_type: &'static str,
    pub permissions: Vec<Permission>,
}

pub async fn check_permissions(
    State(state): State<AppState>,
    axum::Extension(config): axum::Extension<RequiredPermissions>,
    auth_user: AuthUser,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user_id = auth_user
        .claims
        .user_id()
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    // Verify user type matches the requirement
    if auth_user.claims.user_type != config.user_type {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    // For admins, we check the DB for permissions
    if config.user_type == "admin" {
        let repo = PostgresAdministratorRepository::new(state.pool.clone());
        let permissions = repo.get_permissions(user_id).await.map_err(|e| {
            tracing::error!("Failed to fetch permissions for user {}: {}", user_id, e);
            AppError::InternalServerError(e)
        })?;

        // Check if user has Wildcard OR any of the required permissions
        let has_permission = permissions.contains(&Permission::Wildcard)
            || config
                .permissions
                .iter()
                .any(|req| permissions.contains(req));

        if !has_permission {
            return Err(AppError::Forbidden("Insufficient permissions".to_string()));
        }
    } else {
        // For other user types, implement logic as needed (e.g. merchant scopes)
        // Currently fail open/closed depending on design. Fails closed here for safety.
        tracing::warn!("RBAC not implemented for user type: {}", config.user_type);
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    Ok(next.run(request).await)
}
