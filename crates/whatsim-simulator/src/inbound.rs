use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use whatsim_core::{
    Event, EventType, Message, MessageDirection, MessageKind, MessageProvider,
    NormalizedInboundEvent, PayloadDirection, PayloadKind, PayloadSnapshot, WhatsimError,
};
use whatsim_provider_meta::generate::{generate_inbound_text_payload, generate_outbound_response};
use whatsim_storage::InMemoryStore;

/// Orchestrates the full simulation flow: generating provider payloads,
/// persisting domain objects, and optionally forwarding webhooks to a target
/// application.
#[derive(Debug, Clone)]
pub struct SimulationEngine {
    store: InMemoryStore,
    webhook_target: Option<String>,
    webhook_secret: Option<String>,
    http_client: reqwest::Client,
}

impl SimulationEngine {
    pub fn new(
        store: InMemoryStore,
        webhook_target: Option<String>,
        webhook_secret: Option<String>,
    ) -> Self {
        Self {
            store,
            webhook_target,
            webhook_secret,
            http_client: reqwest::Client::new(),
        }
    }

    /// Returns a reference to the underlying store.
    pub fn store(&self) -> &InMemoryStore {
        &self.store
    }

    /// Simulate a full inbound text message flow.
    ///
    /// 1. Generate a Meta Cloud API webhook payload.
    /// 2. Create a payload snapshot.
    /// 3. Create an inbound message record.
    /// 4. Create a normalized inbound event.
    /// 5. Create an event record.
    /// 6. If a webhook target is configured, POST the payload to it.
    /// 7. Record the webhook delivery result as an event.
    /// 8. Update the conversation timestamp.
    /// 9. Return the created message and normalized event.
    pub async fn simulate_inbound_text(
        &self,
        conversation_id: Uuid,
        text: &str,
    ) -> Result<(Message, NormalizedInboundEvent), WhatsimError> {
        let now = Utc::now();

        // Look up the conversation to get phone numbers and contact name.
        let conversation = self
            .store
            .get_conversation(conversation_id)
            .await?
            .ok_or_else(|| {
                WhatsimError::NotFound(format!("conversation {conversation_id}"))
            })?;

        // Generate a unique external message ID.
        let external_message_id = format!("wamid.sim_{}", Uuid::new_v4().simple());

        // 1. Generate Meta webhook payload.
        let webhook_payload = generate_inbound_text_payload(
            &conversation.from_phone,
            &conversation.to_phone,
            conversation.contact_name.as_deref().unwrap_or("Unknown"),
            text,
            &external_message_id,
            now,
        );

        let payload_json = serde_json::to_value(&webhook_payload)
            .map_err(|e| WhatsimError::Internal(e.to_string()))?;

        // 2. Create payload snapshot.
        let snapshot_id = Uuid::new_v4();
        let snapshot = PayloadSnapshot {
            id: snapshot_id,
            conversation_id,
            direction: PayloadDirection::Inbound,
            payload_json: payload_json.clone(),
            created_at: now,
            payload_kind: PayloadKind::MetaWebhook,
        };
        self.store.add_payload_snapshot(snapshot).await?;

        // 3. Create inbound message.
        let message_id = Uuid::new_v4();
        let message = Message {
            id: message_id,
            conversation_id,
            direction: MessageDirection::Inbound,
            kind: MessageKind::Text,
            text: Some(text.to_string()),
            external_message_id: Some(external_message_id.clone()),
            timestamp: now,
            raw_payload_id: Some(snapshot_id),
            provider: MessageProvider::MetaSimulated,
            delivery_status: None,
            metadata: None,
        };
        self.store.add_message(message.clone()).await?;

        // 4. Create normalized inbound event.
        let normalized_event = NormalizedInboundEvent {
            provider: "meta_simulated".to_string(),
            external_message_id: external_message_id.clone(),
            from_phone: conversation.from_phone.clone(),
            to_phone: conversation.to_phone.clone(),
            contact_name: conversation.contact_name.clone(),
            message_type: "text".to_string(),
            text: Some(text.to_string()),
            timestamp: now,
            organization_id: conversation.organization_id.clone(),
            raw_payload: payload_json.clone(),
        };

        // 5. Create event record.
        let event = Event {
            id: Uuid::new_v4(),
            conversation_id,
            event_type: EventType::InboundMessage,
            timestamp: now,
            payload: Some(serde_json::to_value(&normalized_event).unwrap_or_default()),
        };
        self.store.add_event(event).await?;

        // 6 & 7. Forward webhook payload and record delivery result.
        if let Some(ref target_url) = self.webhook_target {
            let delivery_event_type;
            let delivery_payload;

            // Serialize body once so we can sign it.
            let body_bytes = serde_json::to_vec(&webhook_payload)
                .map_err(|e| WhatsimError::Internal(e.to_string()))?;

            let mut request = self
                .http_client
                .post(target_url)
                .header("Content-Type", "application/json");

            // Sign with HMAC-SHA256 if a webhook secret is configured,
            // matching Meta's X-Hub-Signature-256 header format.
            if let Some(ref secret) = self.webhook_secret {
                let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                    .expect("HMAC accepts any key length");
                mac.update(&body_bytes);
                let signature = hex::encode(mac.finalize().into_bytes());
                request = request.header("X-Hub-Signature-256", format!("sha256={signature}"));
            }

            match request
                .body(body_bytes)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        tracing::info!(
                            url = %target_url,
                            status = %status,
                            message_id = %external_message_id,
                            "Webhook delivered successfully"
                        );
                        delivery_event_type = EventType::WebhookDelivery;
                        delivery_payload = serde_json::json!({
                            "url": target_url,
                            "status": status.as_u16(),
                            "success": true,
                        });
                    } else {
                        let body = response.text().await.unwrap_or_default();
                        tracing::warn!(
                            url = %target_url,
                            status = %status,
                            body = %body,
                            message_id = %external_message_id,
                            "Webhook delivery returned non-success status"
                        );
                        delivery_event_type = EventType::WebhookDeliveryFailed;
                        delivery_payload = serde_json::json!({
                            "url": target_url,
                            "status": status.as_u16(),
                            "success": false,
                            "body": body,
                        });
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        url = %target_url,
                        error = %err,
                        message_id = %external_message_id,
                        "Webhook delivery failed"
                    );
                    delivery_event_type = EventType::WebhookDeliveryFailed;
                    delivery_payload = serde_json::json!({
                        "url": target_url,
                        "success": false,
                        "error": err.to_string(),
                    });
                }
            }

            let delivery_event = Event {
                id: Uuid::new_v4(),
                conversation_id,
                event_type: delivery_event_type,
                timestamp: Utc::now(),
                payload: Some(delivery_payload),
            };
            self.store.add_event(delivery_event).await?;
        }

        // 8. Update conversation timestamp.
        self.store
            .update_conversation_timestamp(conversation_id)
            .await?;

        // 9. Return the created message and normalized event.
        Ok((message, normalized_event))
    }

    /// Process an outbound message sent by the target application through the
    /// mock Meta send-message endpoint.
    ///
    /// 1. Find the conversation by the recipient phone number.
    /// 2. Create an outbound message record.
    /// 3. Create a payload snapshot.
    /// 4. Create an event.
    /// 5. Update conversation timestamp.
    /// 6. Return the created message.
    pub async fn process_outbound(
        &self,
        to_phone: &str,
        text: &str,
    ) -> Result<Message, WhatsimError> {
        let now = Utc::now();

        // 1. Find conversation by phone.
        let conversation = self
            .store
            .find_conversation_by_phone(to_phone)
            .await?
            .ok_or_else(|| {
                WhatsimError::NotFound(format!("conversation with phone {to_phone}"))
            })?;

        // Generate a unique external message ID for the outbound message.
        let external_message_id = format!("wamid.sim_out_{}", Uuid::new_v4().simple());

        // Generate the mock Meta outbound response.
        let outbound_response = generate_outbound_response(to_phone, &external_message_id);

        let response_json = serde_json::to_value(&outbound_response)
            .map_err(|e| WhatsimError::Internal(e.to_string()))?;

        // 3. Create payload snapshot.
        let snapshot_id = Uuid::new_v4();
        let snapshot = PayloadSnapshot {
            id: snapshot_id,
            conversation_id: conversation.id,
            direction: PayloadDirection::Outbound,
            payload_json: response_json,
            created_at: now,
            payload_kind: PayloadKind::MockMetaOutbound,
        };
        self.store.add_payload_snapshot(snapshot).await?;

        // 2. Create outbound message.
        let message_id = Uuid::new_v4();
        let message = Message {
            id: message_id,
            conversation_id: conversation.id,
            direction: MessageDirection::Outbound,
            kind: MessageKind::Text,
            text: Some(text.to_string()),
            external_message_id: Some(external_message_id),
            timestamp: now,
            raw_payload_id: Some(snapshot_id),
            provider: MessageProvider::MockMetaOutbound,
            delivery_status: Some("sent".to_string()),
            metadata: None,
        };
        self.store.add_message(message.clone()).await?;

        // 4. Create event.
        let event = Event {
            id: Uuid::new_v4(),
            conversation_id: conversation.id,
            event_type: EventType::OutboundMessage,
            timestamp: now,
            payload: Some(serde_json::json!({
                "to": to_phone,
                "text": text,
                "message_id": message.external_message_id,
            })),
        };
        self.store.add_event(event).await?;

        // 5. Update conversation timestamp.
        self.store
            .update_conversation_timestamp(conversation.id)
            .await?;

        // 6. Return the created message.
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatsim_core::{Conversation, EventType, MessageDirection};

    /// Helper: build a test conversation and insert it into the store.
    async fn setup_conversation(store: &InMemoryStore) -> Conversation {
        let now = Utc::now();
        let conversation = Conversation {
            id: Uuid::new_v4(),
            organization_id: Some("org-test".into()),
            contact_name: Some("Test User".into()),
            from_phone: "+15551112222".into(),
            to_phone: "+15553334444".into(),
            created_at: now,
            updated_at: now,
            metadata: None,
        };
        store
            .create_conversation(conversation.clone())
            .await
            .unwrap();
        conversation
    }

    #[tokio::test]
    async fn test_simulate_inbound_text() {
        let store = InMemoryStore::new();
        let conv = setup_conversation(&store).await;
        let engine = SimulationEngine::new(store.clone(), None, None);

        let (message, normalized_event) = engine
            .simulate_inbound_text(conv.id, "Hello from test")
            .await
            .unwrap();

        // Returned message has correct conversation_id, direction, and text.
        assert_eq!(message.conversation_id, conv.id);
        assert_eq!(message.direction, MessageDirection::Inbound);
        assert_eq!(message.text.as_deref(), Some("Hello from test"));

        // Normalized event has correct fields.
        assert_eq!(normalized_event.from_phone, conv.from_phone);
        assert_eq!(normalized_event.text.as_deref(), Some("Hello from test"));
        assert_eq!(normalized_event.provider, "meta_simulated");

        // Store has the message.
        let messages = store.list_messages(conv.id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);

        // Store has events (at least 1 InboundMessage event).
        let events = store.list_events(conv.id).await.unwrap();
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| e.event_type == EventType::InboundMessage));

        // Store has payload snapshot.
        let snapshot = store
            .get_payload_snapshot(message.raw_payload_id.unwrap())
            .await
            .unwrap();
        assert!(snapshot.is_some());
    }

    #[tokio::test]
    async fn test_simulate_inbound_text_missing_conversation() {
        let store = InMemoryStore::new();
        let engine = SimulationEngine::new(store, None, None);

        let result = engine
            .simulate_inbound_text(Uuid::new_v4(), "ghost message")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, WhatsimError::NotFound(_)),
            "expected NotFound, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_process_outbound() {
        let store = InMemoryStore::new();
        let conv = setup_conversation(&store).await;
        let engine = SimulationEngine::new(store.clone(), None, None);

        let message = engine
            .process_outbound(&conv.from_phone, "Reply from bot")
            .await
            .unwrap();

        // Returned message has direction Outbound and correct text.
        assert_eq!(message.direction, MessageDirection::Outbound);
        assert_eq!(message.text.as_deref(), Some("Reply from bot"));
        assert_eq!(message.conversation_id, conv.id);

        // Store has the outbound message.
        let messages = store.list_messages(conv.id).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message.id);
        assert_eq!(messages[0].direction, MessageDirection::Outbound);

        // Store has events (at least 1 OutboundMessage event).
        let events = store.list_events(conv.id).await.unwrap();
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| e.event_type == EventType::OutboundMessage));
    }

    #[tokio::test]
    async fn test_process_outbound_unknown_phone() {
        let store = InMemoryStore::new();
        let engine = SimulationEngine::new(store, None, None);

        let result = engine
            .process_outbound("+19999999999", "message to nobody")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, WhatsimError::NotFound(_)),
            "expected NotFound, got: {err:?}"
        );
    }
}
