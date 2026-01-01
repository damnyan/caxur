use crate::presentation::handlers::auth;
use axum::{Router, routing::post};

use crate::infrastructure::state::AppState;

/// Auth routes - handles authentication endpoints
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}
