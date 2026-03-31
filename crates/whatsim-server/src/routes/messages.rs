use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use whatsim_core::types::Message;

use crate::errors::AppError;
use crate::state::{AppState, BroadcastEvent};

/// GET /api/conversations/:id/messages
pub async fn list_messages(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, AppError> {
    let messages = state
        .engine
        .store()
        .list_messages(id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(messages))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundTextRequest {
    pub conversation_id: Uuid,
    pub text: String,
}

/// POST /api/messages/inbound-text
pub async fn inbound_text(
    State(state): State<AppState>,
    Json(body): Json<InboundTextRequest>,
) -> Result<Json<Message>, AppError> {
    let (message, _normalized_event) = state
        .engine
        .simulate_inbound_text(body.conversation_id, &body.text)
        .await
        .map_err(AppError::from)?;

    // Broadcast the new message.
    let _ = state.tx.send(BroadcastEvent::NewMessage(message.clone()));

    // Also broadcast conversation updated since the timestamp changed.
    if let Ok(Some(conv)) = state.engine.store().get_conversation(body.conversation_id).await {
        let _ = state.tx.send(BroadcastEvent::ConversationUpdated(conv));
    }

    Ok(Json(message))
}
