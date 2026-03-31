use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A provider-agnostic representation of an inbound message received from a
/// messaging platform (or the local simulator).  Every provider adapter
/// normalises its raw webhook payload into this shape before handing it to the
/// core processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedInboundEvent {
    pub provider: String,
    pub external_message_id: String,
    pub from_phone: String,
    pub to_phone: String,
    pub contact_name: Option<String>,
    /// The message type.  Currently always `"text"`.
    pub message_type: String,
    pub text: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Option<String>,
    pub raw_payload: serde_json::Value,
}
