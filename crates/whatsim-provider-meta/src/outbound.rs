use serde::{Deserialize, Serialize};

use crate::webhook_payload::MetaTextPayload;

/// What a target app sends to the mock Meta send-message endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSendMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messaging_product: Option<String>,
    pub to: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<MetaTextPayload>,
}

/// Response returned by the mock Meta send-message endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSendMessageResponse {
    pub messaging_product: String,
    pub contacts: Vec<MetaSendContact>,
    pub messages: Vec<MetaSendMessageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSendContact {
    pub input: String,
    pub wa_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaSendMessageId {
    pub id: String,
}
