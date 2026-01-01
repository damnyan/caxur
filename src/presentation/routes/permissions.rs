use crate::presentation::handlers::permissions;
use axum::{Router, routing::get};

use crate::infrastructure::state::AppState;

/// Permission routes - handles permission listing
pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(permissions::list_permissions))
}
