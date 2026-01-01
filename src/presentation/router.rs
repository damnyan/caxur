use crate::presentation::handlers;
use crate::presentation::openapi::ApiDoc;
use crate::presentation::routes;
use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::state::AppState;

pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/health", get(handlers::health::health_check))
        .nest("/api/v1/auth", routes::auth::routes())
        .nest("/api/v1/users", routes::users::routes())
        .nest(
            "/api/v1/admin/administrators",
            routes::administrators::routes(),
        )
        .nest("/api/v1/admin/roles", routes::roles::routes())
        .nest("/api/v1/admin/permissions", routes::permissions::routes())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
