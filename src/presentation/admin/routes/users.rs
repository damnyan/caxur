use crate::presentation::admin::handlers::users;
use axum::{Router, routing::get};

use crate::infrastructure::state::AppState;

/// Admin User Management routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(users::list_users))
}
