use std::env;
use tower_http::cors::{Any, CorsLayer};

use axum::http::HeaderValue;

pub fn cors_layer() -> anyhow::Result<CorsLayer> {
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());

    if allowed_origins.is_empty() || allowed_origins == "*" {
        return Ok(CorsLayer::new().allow_origin(Any));
    }

    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .map(|s| s.trim().parse())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Invalid CORS origin: {}", e))?;

    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any))
}
