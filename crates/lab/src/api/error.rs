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

        // RFC 9728: include WWW-Authenticate on 401 responses so MCP clients
        // can discover the authorization server via resource metadata.
        if status == StatusCode::UNAUTHORIZED {
            // IntoResponse has no access to AppState, so we fall back to the
            // default resource URL. The auth middleware in router.rs has access
            // to state and can set a more specific header when needed.
            let www_auth = crate::api::oauth::www_authenticate_value(None);
            let mut response = (status, axum::Json(body)).into_response();
            match axum::http::HeaderValue::from_str(&www_auth) {
                Ok(value) => {
                    response
                        .headers_mut()
                        .insert(axum::http::header::WWW_AUTHENTICATE, value);
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to construct WWW-Authenticate header value"
                    );
                }
            }
            response
        } else {
            (status, axum::Json(body)).into_response()
        }
    }
}
