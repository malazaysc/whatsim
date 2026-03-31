use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Conversation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: Uuid,
    pub organization_id: Option<String>,
    pub contact_name: Option<String>,
    pub from_phone: String,
    pub to_phone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageKind {
    Text,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageProvider {
    MetaSimulated,
    MockMetaOutbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub direction: MessageDirection,
    pub kind: MessageKind,
    pub text: Option<String>,
    pub external_message_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub raw_payload_id: Option<Uuid>,
    pub provider: MessageProvider,
    pub delivery_status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Event
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventType {
    InboundMessage,
    OutboundMessage,
    WebhookDelivery,
    WebhookDeliveryFailed,
    SystemNotice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub payload: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// PayloadSnapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PayloadDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PayloadKind {
    MetaWebhook,
    MockMetaOutbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayloadSnapshot {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub direction: PayloadDirection,
    pub payload_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub payload_kind: PayloadKind,
}
