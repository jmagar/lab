//! Surface-neutral error type for dispatch operations.
//!
//! `ToolError` is the single canonical error type across all three surfaces
//! (MCP, API, CLI). It lives here in `dispatch/` because it is
//! surface-neutral — no surface module should own it.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Error variants that dispatchers can produce on top of SDK errors.
///
/// **Serialization contract:** `Serialize` is hand-written so the `Sdk` variant
/// promotes `sdk_kind` to the top-level `kind` field. `Deserialize` is NOT
/// derived — the derived impl would expect `{"kind":"sdk","sdk_kind":"..."}`,
/// which disagrees with the wire format `{"kind":"auth_failed","message":"..."}`.
/// If you need deserialization, deserialize into `serde_json::Value` and
/// construct variants manually.
#[derive(Debug, Clone)]
pub enum ToolError {
    /// Action name not recognized for this service.
    UnknownAction {
        /// Human-readable message.
        message: String,
        /// Valid action names for this service.
        valid: Vec<String>,
        /// Optional fuzzy suggestion.
        hint: Option<String>,
    },
    /// Required parameter missing.
    MissingParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Parameter present but wrong type or value.
    InvalidParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Multi-instance label not found.
    UnknownInstance {
        /// Human-readable message.
        message: String,
        /// Known instance labels.
        valid: Vec<String>,
    },
    /// Pass-through of an `ApiError::kind()` tag from the SDK.
    Sdk {
        /// Stable kind tag (`auth_failed`, `rate_limited`, …).
        sdk_kind: String,
        /// Human-readable message.
        message: String,
    },
}

impl Serialize for ToolError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::UnknownAction {
                message,
                valid,
                hint,
            } => serde_json::json!({
                "kind": "unknown_action",
                "message": message,
                "valid": valid,
                "hint": hint,
            }),
            Self::MissingParam { message, param } => serde_json::json!({
                "kind": "missing_param",
                "message": message,
                "param": param,
            }),
            Self::InvalidParam { message, param } => serde_json::json!({
                "kind": "invalid_param",
                "message": message,
                "param": param,
            }),
            Self::UnknownInstance { message, valid } => serde_json::json!({
                "kind": "unknown_instance",
                "message": message,
                "valid": valid,
            }),
            Self::Sdk { sdk_kind, message } => serde_json::json!({
                "kind": sdk_kind,
                "message": message,
            }),
        };
        v.serialize(serializer)
    }
}

impl IntoResponse for ToolError {
    fn into_response(self) -> Response {
        let status = match self.kind() {
            "auth_failed" => StatusCode::UNAUTHORIZED,
            "not_found" => StatusCode::NOT_FOUND,
            "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
            "missing_param" | "invalid_param" | "validation_failed" | "confirmation_required" => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            "unknown_action" | "unknown_subaction" | "unknown_instance" => StatusCode::BAD_REQUEST,
            "network_error" | "server_error" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Serialize self directly — byte-identical to the MCP error envelope.
        let body = serde_json::to_value(&self).unwrap_or_else(|_| {
            serde_json::json!({"kind": "internal_error", "message": "error serialization failed"})
        });
        (status, axum::Json(body)).into_response()
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialize to the stable JSON envelope so callers get a machine-readable string.
        match serde_json::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => write!(f, "{self:?}"),
        }
    }
}

impl ToolError {
    /// Canonical stable string tag.
    #[must_use]
    pub const fn kind(&self) -> &str {
        match self {
            Self::UnknownAction { .. } => "unknown_action",
            Self::MissingParam { .. } => "missing_param",
            Self::InvalidParam { .. } => "invalid_param",
            Self::UnknownInstance { .. } => "unknown_instance",
            Self::Sdk { sdk_kind, .. } => sdk_kind.as_str(),
        }
    }

    /// Whether this error represents an internal/fatal failure (ERROR level)
    /// vs a caller/user error (WARN level).
    ///
    /// Per OBSERVABILITY.md:
    /// - WARN: user/caller errors the caller can fix
    /// - ERROR: unhandled/fatal errors requiring operator investigation
    #[must_use]
    pub fn is_internal(&self) -> bool {
        matches!(
            self.kind(),
            "internal_error" | "server_error" | "decode_error"
        )
    }
}

// ── From<ServiceError> for ToolError ─────────────────────────────────────
//
// All SDK error → ToolError conversions live here (not in MCP or HTTP
// surface modules) so both surfaces share one conversion path.
// Each impl is feature-gated to match its service.

#[cfg(feature = "bytestash")]
impl From<lab_apis::bytestash::error::ByteStashError> for ToolError {
    fn from(e: lab_apis::bytestash::error::ByteStashError) -> Self {
        let kind = match &e {
            lab_apis::bytestash::error::ByteStashError::Api(api) => api.kind(),
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "radarr")]
impl From<lab_apis::radarr::error::RadarrError> for ToolError {
    fn from(e: lab_apis::radarr::error::RadarrError) -> Self {
        let kind = match &e {
            lab_apis::radarr::error::RadarrError::Api(api) => api.kind(),
            lab_apis::radarr::error::RadarrError::NotFound { .. } => "not_found",
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "sabnzbd")]
impl From<lab_apis::sabnzbd::error::SabnzbdError> for ToolError {
    fn from(e: lab_apis::sabnzbd::error::SabnzbdError) -> Self {
        let kind = match &e {
            lab_apis::sabnzbd::error::SabnzbdError::Api(api) => api.kind(),
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "unifi")]
impl From<lab_apis::unifi::error::UnifiError> for ToolError {
    fn from(e: lab_apis::unifi::error::UnifiError) -> Self {
        let kind = match &e {
            lab_apis::unifi::error::UnifiError::Api(api) => api.kind(),
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}
