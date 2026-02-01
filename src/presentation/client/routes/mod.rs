pub mod auth;
pub mod users;

use crate::infrastructure::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/users", users::routes())
}
