mod assets;
mod errors;
mod routes;
mod state;

use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;
use tokio::sync::broadcast;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use whatsim_core::config::AppConfig;
use whatsim_simulator::SimulationEngine;
use whatsim_storage::InMemoryStore;

use crate::state::AppState;

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

    // Build the router.
    let app = Router::new()
        // Health check
        .route("/health", get(routes::health::health))
        // API routes
        .route("/api/config", get(routes::config::get_config))
        .route(
            "/api/conversations",
            get(routes::conversations::list_conversations)
                .post(routes::conversations::create_conversation),
        )
        .route(
            "/api/conversations/{id}",
            get(routes::conversations::get_conversation),
        )
        .route(
            "/api/conversations/{id}/messages",
            get(routes::messages::list_messages),
        )
        .route(
            "/api/conversations/{id}/events",
            get(routes::events::list_events),
        )
        .route(
            "/api/messages/inbound-text",
            post(routes::messages::inbound_text),
        )
        .route(
            "/api/mock-meta/messages",
            post(routes::mock_meta::send_message),
        )
        .route("/api/stream", get(routes::stream::event_stream))
        // Static assets / SPA fallback
        .fallback(get(assets::static_handler))
        // Middleware
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Whatsim server starting at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
