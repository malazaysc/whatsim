use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use whatsim_core::errors::WhatsimError;

/// A thin wrapper so we can implement `IntoResponse` for an external type
/// without orphan-rule issues.
pub struct AppError(pub WhatsimError);

impl From<WhatsimError> for AppError {
    fn from(err: WhatsimError) -> Self {
        Self(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let err = &self.0;
        let status = match err {
            WhatsimError::NotFound(_) => StatusCode::NOT_FOUND,
            WhatsimError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            WhatsimError::WebhookDeliveryFailed(_) => StatusCode::BAD_GATEWAY,
            WhatsimError::StorageError(_) | WhatsimError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let body = serde_json::json!({
            "kind": err.kind(),
            "message": err.to_string(),
        });

        (status, axum::Json(body)).into_response()
    }
}
