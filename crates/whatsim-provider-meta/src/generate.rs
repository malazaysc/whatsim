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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generate_inbound_text_payload() {
        let timestamp = Utc.with_ymd_and_hms(2025, 6, 15, 12, 30, 0).unwrap();
        let payload = generate_inbound_text_payload(
            "+1234567890",
            "+0987654321",
            "Alice",
            "Hello, world!",
            "wamid.test123",
            timestamp,
        );

        // Top-level object
        assert_eq!(payload.object, "whatsapp_business_account");

        // One entry with one change
        assert_eq!(payload.entry.len(), 1);
        let entry = &payload.entry[0];
        assert_eq!(entry.changes.len(), 1);

        let change = &entry.changes[0];
        assert_eq!(change.field, "messages");

        let value = &change.value;
        assert_eq!(value.messaging_product, "whatsapp");

        // Contact name matches
        assert_eq!(value.contacts.len(), 1);
        assert_eq!(value.contacts[0].profile.name, "Alice");
        assert_eq!(value.contacts[0].wa_id, "+1234567890");

        // Message fields
        assert_eq!(value.messages.len(), 1);
        let msg = &value.messages[0];
        assert_eq!(msg.from, "+1234567890");
        assert_eq!(msg.message_type, "text");
        assert_eq!(msg.id, "wamid.test123");

        // Text body matches
        let text = msg.text.as_ref().expect("text should be present");
        assert_eq!(text.body, "Hello, world!");

        // Timestamp is the correct unix timestamp string
        assert_eq!(msg.timestamp, timestamp.timestamp().to_string());
    }

    #[test]
    fn test_generate_outbound_response() {
        let response = generate_outbound_response("+5551234", "wamid.out_abc");

        assert_eq!(response.messaging_product, "whatsapp");

        // Contacts has the right wa_id
        assert_eq!(response.contacts.len(), 1);
        assert_eq!(response.contacts[0].wa_id, "+5551234");
        assert_eq!(response.contacts[0].input, "+5551234");

        // Messages has the right id
        assert_eq!(response.messages.len(), 1);
        assert_eq!(response.messages[0].id, "wamid.out_abc");
    }

    #[test]
    fn test_payload_serialization_roundtrip() {
        let timestamp = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let original = generate_inbound_text_payload(
            "+11111111111",
            "+22222222222",
            "Bob",
            "Test message",
            "wamid.roundtrip",
            timestamp,
        );

        let json_str =
            serde_json::to_string(&original).expect("serialization should succeed");
        let deserialized: MetaWebhookPayload =
            serde_json::from_str(&json_str).expect("deserialization should succeed");

        // Verify structural equality after roundtrip
        assert_eq!(deserialized.object, original.object);
        assert_eq!(deserialized.entry.len(), original.entry.len());
        assert_eq!(
            deserialized.entry[0].changes[0].field,
            original.entry[0].changes[0].field
        );
        assert_eq!(
            deserialized.entry[0].changes[0].value.messaging_product,
            original.entry[0].changes[0].value.messaging_product
        );
        assert_eq!(
            deserialized.entry[0].changes[0].value.contacts[0].profile.name,
            original.entry[0].changes[0].value.contacts[0].profile.name
        );
        assert_eq!(
            deserialized.entry[0].changes[0].value.messages[0].from,
            original.entry[0].changes[0].value.messages[0].from
        );
        assert_eq!(
            deserialized.entry[0].changes[0].value.messages[0].text.as_ref().unwrap().body,
            original.entry[0].changes[0].value.messages[0].text.as_ref().unwrap().body
        );

        // Also verify it round-trips through serde_json::Value
        let value_original = serde_json::to_value(&original).unwrap();
        let value_roundtrip = serde_json::to_value(&deserialized).unwrap();
        assert_eq!(value_original, value_roundtrip);
    }
}
