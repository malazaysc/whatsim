use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum WhatsimError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("webhook delivery failed: {0}")]
    WebhookDeliveryFailed(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Convenience alias used throughout the Whatsim codebase.
pub type WhatsimResult<T> = Result<T, WhatsimError>;

impl WhatsimError {
    /// Returns a short classifier string for the error variant, useful for
    /// logging and error responses.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::WebhookDeliveryFailed(_) => "webhook_delivery_failed",
            Self::StorageError(_) => "storage_error",
            Self::Internal(_) => "internal",
        }
    }
}

// Allow converting a plain string into an Internal error for ergonomic `?`
// usage.
impl From<String> for WhatsimError {
    fn from(msg: String) -> Self {
        Self::Internal(msg)
    }
}

impl From<&str> for WhatsimError {
    fn from(msg: &str) -> Self {
        Self::Internal(msg.to_owned())
    }
}

// Serde support so errors can be returned as JSON in HTTP responses.
impl serde::Serialize for WhatsimError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WhatsimError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
