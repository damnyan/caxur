use crate::presentation::client::handlers::users;
use axum::{
    Router,
    routing::{get, post},
};

use crate::infrastructure::state::AppState;

/// Client User routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(users::create_user)).route(
        "/{id}",
        get(users::get_user)
            .put(users::update_user)
            .delete(users::delete_user),
    )
}
