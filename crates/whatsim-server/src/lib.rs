pub mod assets;
pub mod errors;
pub mod routes;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

/// Build the Axum router with all routes and middleware attached.
///
/// This is extracted from `main` so it can be reused in integration tests.
pub fn build_app(app_state: AppState) -> Router {
    Router::new()
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
        .with_state(app_state)
}
