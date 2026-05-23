use axum::{Router, routing::get};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

mod health;
mod users;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health::health))
        .nest("/users", users::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn root() -> &'static str {
    "dearday api"
}
