use axum::Json;
use serde_json::{json, Value};

/// GET /health
pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": "0.1.0",
    }))
}
