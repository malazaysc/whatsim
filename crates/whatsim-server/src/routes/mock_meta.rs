use axum::extract::State;
use axum::Json;

use whatsim_provider_meta::outbound::{MetaSendMessageRequest, MetaSendMessageResponse};
use whatsim_provider_meta::generate::generate_outbound_response;

use crate::errors::AppError;
use crate::state::{AppState, BroadcastEvent};

/// POST /api/mock-meta/messages
///
/// Accepts a Meta Cloud API send-message request (the same payload a target
/// application would send to `graph.facebook.com`), processes the outbound
/// message through the simulation engine, and returns a realistic response.
pub async fn send_message(
    State(state): State<AppState>,
    Json(body): Json<MetaSendMessageRequest>,
) -> Result<Json<MetaSendMessageResponse>, AppError> {
    let to_phone = &body.to;
    let text = body
        .text
        .as_ref()
        .map(|t| t.body.clone())
        .unwrap_or_default();

    let message = state
        .engine
        .process_outbound(to_phone, &text)
        .await
        .map_err(AppError::from)?;

    // Broadcast the new outbound message.
    let _ = state.tx.send(BroadcastEvent::NewMessage(message.clone()));

    // Broadcast conversation updated.
    if let Ok(Some(conv)) = state.engine.store().find_conversation_by_phone(to_phone).await {
        let _ = state.tx.send(BroadcastEvent::ConversationUpdated(conv));
    }

    // Build the mock Meta response using the engine-generated external message
    // id.
    let external_id = message
        .external_message_id
        .as_deref()
        .unwrap_or("unknown");
    let response = generate_outbound_response(to_phone, external_id);

    Ok(Json(response))
}
