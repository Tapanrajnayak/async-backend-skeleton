use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Transaction not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate idempotency key")]
    IdempotencyConflict,

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::IdempotencyConflict => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidStateTransition { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, axum::Json(body)).into_response()
    }
}
