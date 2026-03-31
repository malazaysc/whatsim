use std::net::SocketAddr;

use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

use whatsim_core::config::AppConfig;
use whatsim_server::build_app;
use whatsim_server::state::AppState;
use whatsim_simulator::SimulationEngine;
use whatsim_storage::InMemoryStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables.
    let config = AppConfig::from_env();

    // Initialize tracing with the configured log level.
    let filter = EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Create the in-memory store and simulation engine.
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, config.webhook_target.clone());

    // Create broadcast channel for SSE streaming (capacity 256 events).
    let (tx, _rx) = broadcast::channel(256);

    let app_state = AppState {
        engine,
        config: config.clone(),
        tx,
    };

    let app = build_app(app_state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Whatsim server starting at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
