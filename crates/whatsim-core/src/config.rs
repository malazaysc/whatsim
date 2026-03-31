use std::env;

use serde::{Deserialize, Serialize};

/// Top-level application configuration for Whatsim.
///
/// Values are populated from environment variables via [`AppConfig::from_env`],
/// with sensible defaults for local development.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub webhook_target: Option<String>,
    pub db_path: Option<String>,
    pub enable_persistence: bool,
    pub default_organization_id: Option<String>,
    pub public_base_url: String,
}

impl AppConfig {
    /// Build an [`AppConfig`] from environment variables.
    ///
    /// | Env var | Default |
    /// |---|---|
    /// | `WHATSIM_HOST` | `127.0.0.1` |
    /// | `WHATSIM_PORT` | `3210` |
    /// | `WHATSIM_LOG_LEVEL` | `info` |
    /// | `WHATSIM_WEBHOOK_TARGET` | `None` |
    /// | `WHATSIM_DB_PATH` | `None` |
    /// | `WHATSIM_ENABLE_PERSISTENCE` | `false` |
    /// | `WHATSIM_DEFAULT_ORGANIZATION_ID` | `None` |
    /// | `WHATSIM_PUBLIC_BASE_URL` | `http://127.0.0.1:3210` |
    pub fn from_env() -> Self {
        let host = env::var("WHATSIM_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("WHATSIM_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(3210);
        let log_level = env::var("WHATSIM_LOG_LEVEL").unwrap_or_else(|_| "info".into());
        let webhook_target = env::var("WHATSIM_WEBHOOK_TARGET").ok().filter(|s| !s.is_empty());
        let db_path = env::var("WHATSIM_DB_PATH").ok().filter(|s| !s.is_empty());
        let enable_persistence = env::var("WHATSIM_ENABLE_PERSISTENCE")
            .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        let default_organization_id = env::var("WHATSIM_DEFAULT_ORGANIZATION_ID")
            .ok()
            .filter(|s| !s.is_empty());
        let public_base_url = env::var("WHATSIM_PUBLIC_BASE_URL")
            .unwrap_or_else(|_| format!("http://{}:{}", host, port));

        Self {
            host,
            port,
            log_level,
            webhook_target,
            db_path,
            enable_persistence,
            default_organization_id,
            public_base_url,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3210,
            log_level: "info".into(),
            webhook_target: None,
            db_path: None,
            enable_persistence: false,
            default_organization_id: None,
            public_base_url: "http://127.0.0.1:3210".into(),
        }
    }
}
