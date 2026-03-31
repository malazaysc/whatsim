use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use whatsim_core::errors::WhatsimError;
use whatsim_core::types::Conversation;

use crate::errors::AppError;
use crate::state::{AppState, BroadcastEvent};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub organization_id: Option<String>,
}

/// GET /api/conversations
pub async fn list_conversations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Conversation>>, AppError> {
    let conversations = state
        .engine
        .store()
        .list_conversations(query.organization_id.as_deref())
        .await
        .map_err(AppError::from)?;
    Ok(Json(conversations))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationRequest {
    pub from_phone: String,
    pub contact_name: Option<String>,
    pub organization_id: Option<String>,
    pub to_phone: Option<String>,
}

/// POST /api/conversations
pub async fn create_conversation(
    State(state): State<AppState>,
    Json(body): Json<CreateConversationRequest>,
) -> Result<(axum::http::StatusCode, Json<Conversation>), AppError> {
    let now = Utc::now();
    let conversation = Conversation {
        id: Uuid::new_v4(),
        organization_id: body.organization_id,
        contact_name: body.contact_name,
        from_phone: body.from_phone,
        to_phone: body.to_phone.unwrap_or_else(|| "+15550001234".to_string()),
        created_at: now,
        updated_at: now,
        metadata: None,
    };

    let created = state
        .engine
        .store()
        .create_conversation(conversation)
        .await
        .map_err(AppError::from)?;

    // Broadcast the new conversation event (ignore send errors -- there may
    // be no subscribers yet).
    let _ = state.tx.send(BroadcastEvent::NewConversation(created.clone()));

    Ok((axum::http::StatusCode::CREATED, Json(created)))
}

/// GET /api/conversations/:id
pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Conversation>, AppError> {
    let conversation = state
        .engine
        .store()
        .get_conversation(id)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::from(WhatsimError::NotFound(format!("conversation {id}"))))?;
    Ok(Json(conversation))
}
