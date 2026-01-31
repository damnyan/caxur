use crate::presentation::handlers::administrators;
use axum::{
    Router,
    routing::{get, post},
};

use crate::infrastructure::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(administrators::create_admin))
        .route("/", get(administrators::list_admins))
        .route(
            "/{id}",
            get(administrators::get_admin)
                .put(administrators::update_admin)
                .delete(administrators::delete_admin),
        )
        .route(
            "/{id}/roles",
            post(administrators::attach_admin_roles).delete(administrators::detach_admin_roles),
        )
}
