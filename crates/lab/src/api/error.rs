//! HTTP error handling.
//!
//! `ToolError` from `crate::dispatch::error` is the canonical error type for
//! all surfaces (MCP, API, CLI). Its `IntoResponse` impl lives here (not in
//! `dispatch/`) because HTTP status code mapping is an API surface concern.
//!
//! `ApiError` was a duplicate type that serialized a bare `{kind, message}`
//! envelope, dropping structured fields. It has been removed — use
//! `ToolError` directly in all HTTP handlers.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub use crate::dispatch::error::ToolError;

impl IntoResponse for ToolError {
    fn into_response(self) -> Response {
        let status = match self.kind() {
            "auth_failed" => StatusCode::UNAUTHORIZED,
            "not_found" => StatusCode::NOT_FOUND,
            "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
            "missing_param" | "invalid_param" | "validation_failed" => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            "confirmation_required" => StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_action" | "unknown_subaction" | "unknown_instance" => StatusCode::BAD_REQUEST,
            "network_error" | "server_error" | "upstream_error" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Serialize self directly — byte-identical to the MCP error envelope.
        let body = serde_json::to_value(&self).unwrap_or_else(|_| {
            serde_json::json!({"kind": "internal_error", "message": "error serialization failed"})
        });

        // RFC 9728: WWW-Authenticate on 401 responses requires the resolved
        // resource_url from AppState. IntoResponse has no access to state, so
        // the auth middleware in router.rs is responsible for adding the header.
        // We omit it here rather than advertising a wrong (localhost) URL.
        (status, axum::Json(body)).into_response()
    }
}
