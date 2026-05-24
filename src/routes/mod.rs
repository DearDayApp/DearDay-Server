use axum::{Router, routing::get};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

pub mod auth;
mod health;

pub fn router(state: AppState) -> Router {
    let (prometheus_layer, metric_handle, collector) = crate::init_metrics();
    Router::new()
        .route("/", get(root))
        .route("/health", get(health::health))
        .nest("/auth", auth::router())
        .route(
            "/metrics",
            get(move || async move {
                collector.collect();
                metric_handle.render()
            }),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(prometheus_layer)
        .with_state(state)
}

async fn root() -> &'static str {
    "dearday api"
}
