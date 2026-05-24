use std::sync::OnceLock;

use anyhow::Result;
use axum_prometheus::{PrometheusMetricLayer, metrics_exporter_prometheus::PrometheusHandle};
use metrics_process::Collector;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod routes;
mod state;

pub use config::Config;
pub use state::AppState;

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dearday=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// PrometheusMetricLayer::pair() installs a process-global metrics recorder, which
// panics if called twice. Memoize so tests that spawn the router multiple times
// (one per #[sqlx::test]) reuse the same recorder.
static METRICS: OnceLock<(PrometheusMetricLayer<'static>, PrometheusHandle, Collector)> =
    OnceLock::new();

pub fn init_metrics() -> (PrometheusMetricLayer<'static>, PrometheusHandle, Collector) {
    let (layer, handle, collector) = METRICS.get_or_init(|| {
        let (layer, handle) = PrometheusMetricLayer::pair();
        let collector = Collector::default();
        collector.describe();
        (layer, handle, collector)
    });
    (layer.clone(), handle.clone(), collector.clone())
}

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<()> {
    let state = AppState::new(pool);
    let app = routes::router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
