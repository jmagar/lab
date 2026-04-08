//! HTTP error type — maps `lab_apis::core::ApiError` to structured JSON
//! envelopes with stable `kind` tags matching the MCP surface.
//!
//! Keeping the envelope shape identical between MCP and HTTP means clients
//! can share error-handling logic regardless of transport.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use lab_apis::core::ApiError as SdkError;

/// Result alias for axum handlers.
pub type ApiResult<T> = Result<T, ApiError>;

/// HTTP-layer error wrapping `SdkError` plus dispatcher-layer kinds.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Underlying SDK error.
    #[error(transparent)]
    Sdk(#[from] SdkError),

    /// Requested action not recognized for this service.
    #[error("unknown action: {0}")]
    UnknownAction(String),

    /// Required parameter missing from request body.
    #[error("missing parameter: {0}")]
    MissingParam(&'static str),

    /// Parameter present but wrong type or value.
    #[error("invalid parameter `{param}`: {message}")]
    InvalidParam {
        /// Parameter name.
        param: &'static str,
        /// Human-readable reason.
        message: String,
    },

    /// Multi-instance label not found.
    #[error("unknown instance: {0}")]
    UnknownInstance(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// Canonical stable string tag for this error. Identical to the tag
    /// used in MCP error envelopes.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Sdk(e) => e.kind(),
            Self::UnknownAction(_) => "unknown_action",
            Self::MissingParam(_) => "missing_param",
            Self::InvalidParam { .. } => "invalid_param",
            Self::UnknownInstance(_) => "unknown_instance",
            Self::Internal(_) => "internal_error",
        }
    }

    /// HTTP status code for this error kind.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self.kind() {
            "auth_failed" => StatusCode::UNAUTHORIZED,
            "not_found" => StatusCode::NOT_FOUND,
            "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
            "validation_failed" | "missing_param" | "invalid_param" => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            "unknown_action" | "unknown_instance" => StatusCode::BAD_REQUEST,
            "network_error" | "server_error" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = json!({
            "kind": self.kind(),
            "message": self.to_string(),
        });
        (status, Json(body)).into_response()
    }
}
