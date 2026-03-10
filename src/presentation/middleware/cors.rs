use std::env;
use tower_http::cors::{Any, CorsLayer};

use axum::http::HeaderValue;

pub fn cors_layer() -> anyhow::Result<CorsLayer> {
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());

    let mut layer = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ])
        .expose_headers([
            axum::http::header::CONTENT_DISPOSITION,
            axum::http::header::CONTENT_LENGTH,
        ]);

    if allowed_origins.is_empty() || allowed_origins == "*" {
        layer = layer.allow_origin(Any);
    } else {
        let origins: Vec<HeaderValue> = allowed_origins
            .split(',')
            .map(|s| s.trim().parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Invalid CORS origin: {}", e))?;
        layer = layer.allow_origin(origins);
    }

    Ok(layer)
}
