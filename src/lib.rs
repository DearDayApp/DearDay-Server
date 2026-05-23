use anyhow::Result;
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

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<()> {
    let state = AppState::new(pool);
    let app = routes::router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
