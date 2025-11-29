use crate::infrastructure::db::DbPool;
use crate::presentation::openapi::ApiDoc;
use crate::presentation::routes;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn app(pool: DbPool) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1/auth", routes::auth::routes())
        .nest("/api/v1/users", routes::users::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
