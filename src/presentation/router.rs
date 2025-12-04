use crate::infrastructure::db::DbPool;
use crate::presentation::handlers;
use crate::presentation::openapi::ApiDoc;
use crate::presentation::routes;
use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn app(pool: DbPool) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(handlers::health::health_check))
        .nest("/api/v1/auth", routes::auth::routes())
        .nest("/api/v1/users", routes::users::routes())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
