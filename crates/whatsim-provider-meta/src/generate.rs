use chrono::{DateTime, Utc};

use crate::outbound::{MetaSendContact, MetaSendMessageId, MetaSendMessageResponse};
use crate::webhook_payload::{
    MetaContactProfile, MetaTextPayload, MetaWebhookChange, MetaWebhookContact,
    MetaWebhookEntry, MetaWebhookMessage, MetaWebhookMetadata, MetaWebhookPayload,
    MetaWebhookValue,
};

/// Generate a realistic Meta Cloud API webhook payload for an inbound text
/// message.
pub fn generate_inbound_text_payload(
    from_phone: &str,
    to_phone: &str,
    contact_name: &str,
    text: &str,
    message_id: &str,
    timestamp: DateTime<Utc>,
) -> MetaWebhookPayload {
    MetaWebhookPayload {
        object: "whatsapp_business_account".to_string(),
        entry: vec![MetaWebhookEntry {
            id: "sim_biz_001".to_string(),
            changes: vec![MetaWebhookChange {
                value: MetaWebhookValue {
                    messaging_product: "whatsapp".to_string(),
                    metadata: MetaWebhookMetadata {
                        display_phone_number: to_phone.to_string(),
                        phone_number_id: "sim_phone_001".to_string(),
                    },
                    contacts: vec![MetaWebhookContact {
                        profile: MetaContactProfile {
                            name: contact_name.to_string(),
                        },
                        wa_id: from_phone.to_string(),
                    }],
                    messages: vec![MetaWebhookMessage {
                        from: from_phone.to_string(),
                        id: message_id.to_string(),
                        timestamp: timestamp.timestamp().to_string(),
                        message_type: "text".to_string(),
                        text: Some(MetaTextPayload {
                            body: text.to_string(),
                        }),
                    }],
                },
                field: "messages".to_string(),
            }],
        }],
    }
}

/// Generate a mock Meta send-message API response.
pub fn generate_outbound_response(
    to_phone: &str,
    message_id: &str,
) -> MetaSendMessageResponse {
    MetaSendMessageResponse {
        messaging_product: "whatsapp".to_string(),
        contacts: vec![MetaSendContact {
            input: to_phone.to_string(),
            wa_id: to_phone.to_string(),
        }],
        messages: vec![MetaSendMessageId {
            id: message_id.to_string(),
        }],
    }
}
