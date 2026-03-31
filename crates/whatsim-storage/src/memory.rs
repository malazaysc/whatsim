use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use whatsim_core::errors::{WhatsimError, WhatsimResult};
use whatsim_core::types::{Conversation, Event, Message, PayloadSnapshot};

// ---------------------------------------------------------------------------
// Inner (non-thread-safe) store
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct InnerStore {
    conversations: HashMap<Uuid, Conversation>,
    /// Messages keyed by conversation_id.
    messages: HashMap<Uuid, Vec<Message>>,
    /// Events keyed by conversation_id.
    events: HashMap<Uuid, Vec<Event>>,
    payload_snapshots: HashMap<Uuid, PayloadSnapshot>,
    /// Maps a phone number to the conversation it belongs to, so that
    /// outbound messages can be matched back to conversations.
    phone_to_conversation: HashMap<String, Uuid>,
}

// ---------------------------------------------------------------------------
// Public thread-safe wrapper
// ---------------------------------------------------------------------------

/// A thread-safe, in-memory store for all Whatsim domain objects.
///
/// All methods are `async` so the interface stays compatible with a future
/// persistent (e.g. SQLite) implementation -- even though the in-memory
/// operations complete synchronously.
#[derive(Debug, Clone)]
pub struct InMemoryStore {
    inner: Arc<RwLock<InnerStore>>,
}

impl InMemoryStore {
    /// Create a new, empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerStore::default())),
        }
    }

    // -- Conversations ------------------------------------------------------

    /// Insert a new conversation and register phone-to-conversation mappings.
    pub async fn create_conversation(
        &self,
        conversation: Conversation,
    ) -> WhatsimResult<Conversation> {
        let mut store = self.inner.write().await;

        if store.conversations.contains_key(&conversation.id) {
            return Err(WhatsimError::InvalidInput(format!(
                "conversation {} already exists",
                conversation.id
            )));
        }

        // Register both phone numbers so lookups work for either side.
        store
            .phone_to_conversation
            .insert(conversation.from_phone.clone(), conversation.id);
        store
            .phone_to_conversation
            .insert(conversation.to_phone.clone(), conversation.id);

        store
            .conversations
            .insert(conversation.id, conversation.clone());

        Ok(conversation)
    }

    /// Look up a conversation by its id.
    pub async fn get_conversation(&self, id: Uuid) -> WhatsimResult<Option<Conversation>> {
        let store = self.inner.read().await;
        Ok(store.conversations.get(&id).cloned())
    }

    /// Return all conversations, optionally filtered by `organization_id`.
    ///
    /// Results are sorted by `updated_at` descending (most recently updated
    /// first).
    pub async fn list_conversations(
        &self,
        organization_id: Option<&str>,
    ) -> WhatsimResult<Vec<Conversation>> {
        let store = self.inner.read().await;

        let mut conversations: Vec<Conversation> = store
            .conversations
            .values()
            .filter(|c| match organization_id {
                Some(org) => c.organization_id.as_deref() == Some(org),
                None => true,
            })
            .cloned()
            .collect();

        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(conversations)
    }

    /// Touch the `updated_at` timestamp on a conversation.
    pub async fn update_conversation_timestamp(&self, id: Uuid) -> WhatsimResult<()> {
        let mut store = self.inner.write().await;

        let conversation = store
            .conversations
            .get_mut(&id)
            .ok_or_else(|| WhatsimError::NotFound(format!("conversation {id} not found")))?;

        conversation.updated_at = Utc::now();

        Ok(())
    }

    /// Find a conversation by either participant's phone number.
    pub async fn find_conversation_by_phone(
        &self,
        phone: &str,
    ) -> WhatsimResult<Option<Conversation>> {
        let store = self.inner.read().await;

        let conversation = store
            .phone_to_conversation
            .get(phone)
            .and_then(|id| store.conversations.get(id))
            .cloned();

        Ok(conversation)
    }

    // -- Messages -----------------------------------------------------------

    /// Append a message to its conversation's message list.
    pub async fn add_message(&self, message: Message) -> WhatsimResult<Message> {
        let mut store = self.inner.write().await;

        if !store.conversations.contains_key(&message.conversation_id) {
            return Err(WhatsimError::NotFound(format!(
                "conversation {} not found",
                message.conversation_id
            )));
        }

        store
            .messages
            .entry(message.conversation_id)
            .or_default()
            .push(message.clone());

        Ok(message)
    }

    /// Return all messages for a conversation, in insertion order.
    pub async fn list_messages(&self, conversation_id: Uuid) -> WhatsimResult<Vec<Message>> {
        let store = self.inner.read().await;
        Ok(store
            .messages
            .get(&conversation_id)
            .cloned()
            .unwrap_or_default())
    }

    // -- Events -------------------------------------------------------------

    /// Append an event to its conversation's event list.
    pub async fn add_event(&self, event: Event) -> WhatsimResult<Event> {
        let mut store = self.inner.write().await;

        if !store.conversations.contains_key(&event.conversation_id) {
            return Err(WhatsimError::NotFound(format!(
                "conversation {} not found",
                event.conversation_id
            )));
        }

        store
            .events
            .entry(event.conversation_id)
            .or_default()
            .push(event.clone());

        Ok(event)
    }

    /// Return all events for a conversation, in insertion order.
    pub async fn list_events(&self, conversation_id: Uuid) -> WhatsimResult<Vec<Event>> {
        let store = self.inner.read().await;
        Ok(store
            .events
            .get(&conversation_id)
            .cloned()
            .unwrap_or_default())
    }

    // -- Payload snapshots --------------------------------------------------

    /// Store a raw payload snapshot.
    pub async fn add_payload_snapshot(
        &self,
        snapshot: PayloadSnapshot,
    ) -> WhatsimResult<PayloadSnapshot> {
        let mut store = self.inner.write().await;

        store
            .payload_snapshots
            .insert(snapshot.id, snapshot.clone());

        Ok(snapshot)
    }

    /// Retrieve a payload snapshot by its id.
    pub async fn get_payload_snapshot(
        &self,
        id: Uuid,
    ) -> WhatsimResult<Option<PayloadSnapshot>> {
        let store = self.inner.read().await;
        Ok(store.payload_snapshots.get(&id).cloned())
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use whatsim_core::types::{
        EventType, MessageDirection, MessageKind, MessageProvider, PayloadDirection, PayloadKind,
    };

    fn sample_conversation() -> Conversation {
        let now = Utc::now();
        Conversation {
            id: Uuid::new_v4(),
            organization_id: Some("org-1".into()),
            contact_name: Some("Alice".into()),
            from_phone: "+1234567890".into(),
            to_phone: "+0987654321".into(),
            created_at: now,
            updated_at: now,
            metadata: None,
        }
    }

    fn sample_message(conversation_id: Uuid) -> Message {
        Message {
            id: Uuid::new_v4(),
            conversation_id,
            direction: MessageDirection::Inbound,
            kind: MessageKind::Text,
            text: Some("hello".into()),
            external_message_id: None,
            timestamp: Utc::now(),
            raw_payload_id: None,
            provider: MessageProvider::MetaSimulated,
            delivery_status: None,
            metadata: None,
        }
    }

    fn sample_event(conversation_id: Uuid) -> Event {
        Event {
            id: Uuid::new_v4(),
            conversation_id,
            event_type: EventType::InboundMessage,
            timestamp: Utc::now(),
            payload: None,
        }
    }

    fn sample_payload_snapshot(conversation_id: Uuid) -> PayloadSnapshot {
        PayloadSnapshot {
            id: Uuid::new_v4(),
            conversation_id,
            direction: PayloadDirection::Inbound,
            payload_json: serde_json::json!({"test": true}),
            created_at: Utc::now(),
            payload_kind: PayloadKind::MetaWebhook,
        }
    }

    #[tokio::test]
    async fn create_and_get_conversation() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let id = conv.id;

        store.create_conversation(conv.clone()).await.unwrap();

        let fetched = store.get_conversation(id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, id);
    }

    #[tokio::test]
    async fn duplicate_conversation_rejected() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();

        store.create_conversation(conv.clone()).await.unwrap();
        let result = store.create_conversation(conv).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_conversations_filtered_and_sorted() {
        let store = InMemoryStore::new();

        let mut c1 = sample_conversation();
        c1.organization_id = Some("org-a".into());
        c1.from_phone = "+111".into();
        c1.to_phone = "+222".into();
        c1.updated_at = Utc::now() - chrono::Duration::seconds(10);

        let mut c2 = sample_conversation();
        c2.organization_id = Some("org-a".into());
        c2.from_phone = "+333".into();
        c2.to_phone = "+444".into();
        c2.updated_at = Utc::now();

        let mut c3 = sample_conversation();
        c3.organization_id = Some("org-b".into());
        c3.from_phone = "+555".into();
        c3.to_phone = "+666".into();

        store.create_conversation(c1.clone()).await.unwrap();
        store.create_conversation(c2.clone()).await.unwrap();
        store.create_conversation(c3.clone()).await.unwrap();

        let all = store.list_conversations(None).await.unwrap();
        assert_eq!(all.len(), 3);

        let org_a = store.list_conversations(Some("org-a")).await.unwrap();
        assert_eq!(org_a.len(), 2);
        // Most recently updated should come first.
        assert_eq!(org_a[0].id, c2.id);
        assert_eq!(org_a[1].id, c1.id);
    }

    #[tokio::test]
    async fn find_conversation_by_phone() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let id = conv.id;
        let from = conv.from_phone.clone();
        let to = conv.to_phone.clone();

        store.create_conversation(conv).await.unwrap();

        let found_from = store.find_conversation_by_phone(&from).await.unwrap();
        assert_eq!(found_from.unwrap().id, id);

        let found_to = store.find_conversation_by_phone(&to).await.unwrap();
        assert_eq!(found_to.unwrap().id, id);

        let not_found = store
            .find_conversation_by_phone("+0000000000")
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn add_and_list_messages() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let cid = conv.id;
        store.create_conversation(conv).await.unwrap();

        let m1 = sample_message(cid);
        let m2 = sample_message(cid);

        store.add_message(m1.clone()).await.unwrap();
        store.add_message(m2.clone()).await.unwrap();

        let msgs = store.list_messages(cid).await.unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, m1.id);
        assert_eq!(msgs[1].id, m2.id);
    }

    #[tokio::test]
    async fn add_message_to_missing_conversation_fails() {
        let store = InMemoryStore::new();
        let msg = sample_message(Uuid::new_v4());
        let result = store.add_message(msg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_and_list_events() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let cid = conv.id;
        store.create_conversation(conv).await.unwrap();

        let e1 = sample_event(cid);
        store.add_event(e1.clone()).await.unwrap();

        let events = store.list_events(cid).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, e1.id);
    }

    #[tokio::test]
    async fn add_and_get_payload_snapshot() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let cid = conv.id;
        store.create_conversation(conv).await.unwrap();

        let snap = sample_payload_snapshot(cid);
        let snap_id = snap.id;

        store.add_payload_snapshot(snap).await.unwrap();

        let fetched = store.get_payload_snapshot(snap_id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, snap_id);

        let missing = store
            .get_payload_snapshot(Uuid::new_v4())
            .await
            .unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn update_conversation_timestamp() {
        let store = InMemoryStore::new();
        let conv = sample_conversation();
        let id = conv.id;
        let original_updated_at = conv.updated_at;

        store.create_conversation(conv).await.unwrap();

        // Small sleep to ensure the timestamp advances.
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        store.update_conversation_timestamp(id).await.unwrap();

        let fetched = store.get_conversation(id).await.unwrap().unwrap();
        assert!(fetched.updated_at > original_updated_at);
    }
}
