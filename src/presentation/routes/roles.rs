use crate::presentation::handlers::roles;
use axum::{
    Router,
    routing::{get, post},
};

use crate::infrastructure::state::AppState;

/// Role routes - handles role CRUD operations and permission management
pub fn routes() -> Router<AppState> {
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
}
