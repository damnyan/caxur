use crate::domain::permissions::Permission;
use crate::presentation::handlers::roles;
use crate::presentation::middleware::auth::{RequiredPermissions, check_permissions};
use axum::{
    Extension, Router, middleware,
    routing::{get, post},
};

use crate::infrastructure::state::AppState;

/// Role routes - handles role CRUD operations and permission management
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(roles::create_role).get(roles::list_roles))
        .route(
            "/{id}",
            get(roles::get_role)
                .put(roles::update_role)
                .delete(roles::delete_role),
        )
        .route(
            "/{id}/permissions",
            post(roles::attach_permission)
                .get(roles::get_role_permissions)
                .delete(roles::detach_permission),
        )
        .route_layer(middleware::from_fn_with_state(state, check_permissions))
        .route_layer(Extension(RequiredPermissions {
            user_type: "admin",
            permissions: vec![Permission::RoleManagement],
        }))
}
