use crate::presentation::handlers::users;
use axum::{
    Router,
    routing::{get, post},
};

use crate::infrastructure::state::AppState;

/// User routes - handles user CRUD operations
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(users::create_user).get(users::list_users))
        .route(
            "/{id}",
            get(users::get_user)
                .put(users::update_user)
                .delete(users::delete_user),
        )
}
