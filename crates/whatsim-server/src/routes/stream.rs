use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_core::Stream;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::state::{AppState, BroadcastEvent};

/// GET /api/stream
///
/// Server-Sent Events endpoint. Clients subscribe to receive real-time updates
/// for new messages and conversation changes.
pub async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let (event_type, data) = match &event {
                BroadcastEvent::NewMessage(msg) => {
                    ("message", serde_json::to_string(msg).unwrap_or_default())
                }
                BroadcastEvent::NewConversation(conv) => {
                    ("conversation", serde_json::to_string(conv).unwrap_or_default())
                }
                BroadcastEvent::ConversationUpdated(conv) => (
                    "conversation_updated",
                    serde_json::to_string(conv).unwrap_or_default(),
                ),
            };
            Some(Ok(Event::default().event(event_type).data(data)))
        }
        Err(_) => None, // lagged receiver -- skip
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
