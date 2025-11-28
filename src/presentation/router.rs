use crate::infrastructure::db::DbPool;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

pub fn app(pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/users", axum::routing::post(crate::presentation::handlers::users::create_user))
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
