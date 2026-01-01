use crate::infrastructure::auth::JwtAuthService;
use crate::infrastructure::db::DbPool;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub auth_service: Arc<JwtAuthService>,
}

impl AppState {
    pub fn new(pool: DbPool, auth_service: Arc<JwtAuthService>) -> Self {
        Self { pool, auth_service }
    }
}

impl axum::extract::FromRef<AppState> for DbPool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<JwtAuthService> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth_service.clone()
    }
}
