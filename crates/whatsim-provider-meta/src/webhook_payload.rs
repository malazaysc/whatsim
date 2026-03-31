use serde::{Deserialize, Serialize};

/// The top-level webhook payload as delivered by Meta Cloud API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookPayload {
    /// Always `"whatsapp_business_account"`.
    pub object: String,
    pub entry: Vec<MetaWebhookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookEntry {
    /// Business account ID.
    pub id: String,
    pub changes: Vec<MetaWebhookChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookChange {
    pub value: MetaWebhookValue,
    /// Always `"messages"`.
    pub field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookValue {
    pub messaging_product: String,
    pub metadata: MetaWebhookMetadata,
    #[serde(default)]
    pub contacts: Vec<MetaWebhookContact>,
    #[serde(default)]
    pub messages: Vec<MetaWebhookMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookContact {
    pub profile: MetaContactProfile,
    pub wa_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaContactProfile {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWebhookMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<MetaTextPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTextPayload {
    pub body: String,
}
