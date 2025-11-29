use crate::infrastructure::db::DbPool;
use crate::presentation::handlers::auth;
use axum::{Router, routing::post};

/// Auth routes - handles authentication endpoints
pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh_token))
}
