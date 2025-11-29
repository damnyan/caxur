use crate::infrastructure::db::DbPool;
use crate::presentation::openapi::ApiDoc;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn app(pool: DbPool) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(|| async { "ok" }))
        .route(
            "/users",
            post(crate::presentation::handlers::users::create_user)
                .get(crate::presentation::handlers::users::list_users),
        )
        .route(
            "/users/{id}",
            get(crate::presentation::handlers::users::get_user)
                .put(crate::presentation::handlers::users::update_user)
                .delete(crate::presentation::handlers::users::delete_user),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
