use std::env;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "".to_string());

    if allowed_origins.is_empty() || allowed_origins == "*" {
        return CorsLayer::new().allow_origin(Any);
    }

    let origins: Vec<_> = allowed_origins
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any)
}
