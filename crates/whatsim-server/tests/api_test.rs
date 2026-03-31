use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tokio::sync::broadcast;
use tower::ServiceExt;

use whatsim_core::config::AppConfig;
use whatsim_server::build_app;
use whatsim_server::state::AppState;
use whatsim_simulator::SimulationEngine;
use whatsim_storage::InMemoryStore;

/// Build a fresh app + state for each test.
fn test_app() -> axum::Router {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };
    build_app(state)
}

/// Helper: read the full response body as bytes and parse to JSON.
async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("failed to read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("body is not valid JSON")
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint() {
    let app = test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_create_and_list_conversations() {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };

    // Create a conversation.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "fromPhone": "+15551112222",
                        "contactName": "Test User"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    assert!(created["id"].is_string());
    assert_eq!(created["fromPhone"], "+15551112222");
    assert_eq!(created["contactName"], "Test User");

    // List conversations and verify the one we created is there.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/conversations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    let list = list.as_array().expect("should be an array");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["fromPhone"], "+15551112222");
}

#[tokio::test]
async fn test_get_conversation() {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };

    // Create a conversation.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "fromPhone": "+15559998888",
                        "contactName": "Get Test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap();

    // Get the conversation by ID.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched = body_json(response).await;
    assert_eq!(fetched["id"], id);
    assert_eq!(fetched["contactName"], "Get Test");
}

#[tokio::test]
async fn test_inbound_text_flow() {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };

    // Create a conversation.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "fromPhone": "+15550001111",
                        "contactName": "Inbound Test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let created = body_json(response).await;
    let conversation_id = created["id"].as_str().unwrap().to_string();

    // Send an inbound text message.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/messages/inbound-text")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "conversationId": conversation_id,
                        "text": "Hello from simulator"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let message = body_json(response).await;
    assert_eq!(message["direction"], "inbound");
    assert_eq!(message["text"], "Hello from simulator");
    assert_eq!(message["conversationId"], conversation_id);

    // List messages for the conversation -- should contain the inbound message.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{conversation_id}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let messages = body_json(response).await;
    let messages = messages.as_array().expect("should be an array");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["text"], "Hello from simulator");
}

#[tokio::test]
async fn test_mock_meta_outbound_flow() {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };

    // Create a conversation.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "fromPhone": "+15550002222",
                        "contactName": "Outbound Test"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let created = body_json(response).await;
    let conversation_id = created["id"].as_str().unwrap().to_string();

    // Send an outbound message via mock-meta endpoint (by from_phone since
    // process_outbound looks up by phone).
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/mock-meta/messages")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "to": "+15550002222",
                        "type": "text",
                        "text": { "body": "Reply from bot" }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let meta_response = body_json(response).await;
    assert_eq!(meta_response["messaging_product"], "whatsapp");
    assert!(!meta_response["messages"][0]["id"].as_str().unwrap().is_empty());

    // List messages for the conversation -- should contain the outbound message.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{conversation_id}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let messages = body_json(response).await;
    let messages = messages.as_array().expect("should be an array");
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["text"], "Reply from bot");
    assert_eq!(messages[0]["direction"], "outbound");
}

#[tokio::test]
async fn test_full_round_trip() {
    let store = InMemoryStore::new();
    let engine = SimulationEngine::new(store, None);
    let (tx, _rx) = broadcast::channel(256);
    let state = AppState {
        engine,
        config: AppConfig::default(),
        tx,
    };

    // 1. Create a conversation.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/conversations")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "fromPhone": "+15550003333",
                        "contactName": "Round Trip"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let conversation_id = created["id"].as_str().unwrap().to_string();

    // 2. Send an inbound text.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/messages/inbound-text")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "conversationId": conversation_id,
                        "text": "Hi there"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let inbound_msg = body_json(response).await;
    assert_eq!(inbound_msg["direction"], "inbound");

    // 3. Send an outbound via mock-meta.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/mock-meta/messages")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "to": "+15550003333",
                        "type": "text",
                        "text": { "body": "Bot reply" }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. List messages -- verify both are present with correct directions.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{conversation_id}/messages"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let messages = body_json(response).await;
    let messages = messages.as_array().expect("should be an array");
    assert_eq!(messages.len(), 2);

    // First message is inbound, second is outbound.
    assert_eq!(messages[0]["direction"], "inbound");
    assert_eq!(messages[0]["text"], "Hi there");
    assert_eq!(messages[1]["direction"], "outbound");
    assert_eq!(messages[1]["text"], "Bot reply");

    // 5. Verify events were recorded.
    let app = build_app(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/conversations/{conversation_id}/events"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let events = body_json(response).await;
    let events = events.as_array().expect("should be an array");
    // At least 2 events: one inbound, one outbound.
    assert!(
        events.len() >= 2,
        "expected at least 2 events, got {}",
        events.len()
    );
}
