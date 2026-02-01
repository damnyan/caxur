use crate::presentation::admin::handlers::auth;
use axum::{Router, routing::post};

use crate::infrastructure::state::AppState;

/// Admin Auth routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/login", post(auth::admin_login))
}
