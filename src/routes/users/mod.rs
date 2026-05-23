use axum::{routing::get, Router};

use crate::state::AppState;

mod handlers;
mod queries;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list).post(handlers::create))
        .route("/{id}", get(handlers::get_one))
}
