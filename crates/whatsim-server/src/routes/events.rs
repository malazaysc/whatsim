use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use whatsim_core::types::Event;

use crate::errors::AppError;
use crate::state::AppState;

/// GET /api/conversations/:id/events
pub async fn list_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Event>>, AppError> {
    let events = state
        .engine
        .store()
        .list_events(id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(events))
}
