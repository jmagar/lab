//! Structured JSON envelopes returned by every MCP tool dispatch.
//! Shape is identical to what the HTTP API emits (see `api/error.rs`)
//! so clients can share error-handling logic across transports.

use serde::{Deserialize, Serialize};

/// Successful tool result wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolEnvelope<T> {
    /// The tool's result payload.
    pub data: T,
}

impl<T> ToolEnvelope<T> {
    /// Wrap a value in an envelope.
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

/// Error variants that MCP dispatchers can produce on top of SDK errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
        #[serde(rename = "sdk_kind")]
        sdk_kind: String,
        /// Human-readable message.
        message: String,
    },
}

impl ToolError {
    /// Canonical stable string tag.
    #[must_use]
    pub fn kind(&self) -> &str {
        match self {
            Self::UnknownAction { .. } => "unknown_action",
            Self::MissingParam { .. } => "missing_param",
            Self::InvalidParam { .. } => "invalid_param",
            Self::UnknownInstance { .. } => "unknown_instance",
            Self::Sdk { sdk_kind, .. } => sdk_kind.as_str(),
        }
    }
}
