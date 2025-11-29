use crate::infrastructure::db::DbPool;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub fn app(pool: DbPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/users",
            post(crate::presentation::handlers::users::create_user)
                .get(crate::presentation::handlers::users::list_users),
        )
        .route(
            "/users/:id",
            get(crate::presentation::handlers::users::get_user)
                .put(crate::presentation::handlers::users::update_user)
                .delete(crate::presentation::handlers::users::delete_user),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
