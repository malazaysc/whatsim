use tokio::sync::broadcast;

use whatsim_core::config::AppConfig;
use whatsim_core::types::{Conversation, Message};
use whatsim_simulator::SimulationEngine;

/// Events broadcast to SSE/WebSocket subscribers.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "camelCase")]
pub enum BroadcastEvent {
    NewMessage(Message),
    NewConversation(Conversation),
    ConversationUpdated(Conversation),
}

/// Shared application state passed to all Axum handlers via `State`.
#[derive(Clone)]
pub struct AppState {
    pub engine: SimulationEngine,
    pub config: AppConfig,
    pub tx: broadcast::Sender<BroadcastEvent>,
}
