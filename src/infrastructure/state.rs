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
