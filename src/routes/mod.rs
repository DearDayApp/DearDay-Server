use axum::{Router, routing::get};
use axum_prometheus::metrics;
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

pub mod auth;
mod health;

pub fn router(state: AppState) -> Router {
    let (prometheus_layer, metric_handle, collector) = crate::init_metrics();
    let metrics_db = state.db.clone();
    Router::new()
        .route("/", get(root))
        .route("/health", get(health::health))
        .nest("/auth", auth::router())
        .route(
            "/metrics",
            get(move || async move {
                refresh_business_metrics(&metrics_db).await;
                collector.collect();
                metric_handle.render()
            }),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(prometheus_layer)
        .with_state(state)
}

async fn refresh_business_metrics(db: &PgPool) {
    match sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE NOT is_deleted")
        .fetch_one(db)
        .await
    {
        Ok(count) => {
            metrics::gauge!("dearday_users_active").set(count.unwrap_or(0) as f64);
        }
        Err(err) => {
            tracing::warn!("failed to refresh user count metric: {err}");
        }
    }
}

async fn root() -> &'static str {
    "dearday api"
}
