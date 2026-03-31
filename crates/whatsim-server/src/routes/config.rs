use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::state::AppState;

/// GET /api/config
///
/// Returns a sanitised view of the running configuration. Sensitive values
/// (e.g. full webhook URL) are reduced to a boolean flag.
pub async fn get_config(State(state): State<AppState>) -> Json<Value> {
    let cfg = &state.config;
    Json(json!({
        "publicBaseUrl": cfg.public_base_url,
        "webhookTargetConfigured": cfg.webhook_target.is_some(),
        "defaultOrganizationId": cfg.default_organization_id,
        "enablePersistence": cfg.enable_persistence,
    }))
}
