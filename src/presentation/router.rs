use crate::presentation::admin;
use crate::presentation::client;
use crate::presentation::middleware;
use crate::presentation::openapi::ApiDoc;
use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::state::AppState;

pub fn app(state: AppState) -> anyhow::Result<Router> {
    Ok(Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(
                    utoipa_swagger_ui::Config::new(["/api-docs/openapi.json"])
                        .deep_linking(true)
                        .default_models_expand_depth(-1)
                        .display_operation_id(true),
                ),
        )
        .route("/health", get(client::handlers::health::health_check))
        // Client routes (Auth, Users) nested under /api/v1
        .nest("/api/v1", client::routes::routes())
        // Admin routes nested under /api/v1/admin
        .nest("/api/v1/admin", admin::routes::routes(state.clone()))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::cors::cors_layer()?)
        .layer(middleware::rate_limit::rate_limit_layer()?)
        .with_state(state))
}
