//! Helper constructors for [`ToolError`] envelopes. Dispatchers should
//! prefer these over building variants inline — keeps envelope shape
//! consistent across services.
#![allow(dead_code)]

use crate::dispatch::error::ToolError;

/// Build an `unknown_action` envelope with a list of valid actions.
#[must_use]
pub fn unknown_action(service: &str, action: &str, valid: Vec<String>) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `{service}`"),
        valid,
        hint: None,
    }
}

/// Build a `missing_param` envelope.
#[must_use]
pub fn missing_param(param: &str) -> ToolError {
    ToolError::MissingParam {
        message: format!("missing required parameter `{param}`"),
        param: param.to_string(),
    }
}

/// Build an `invalid_param` envelope.
#[must_use]
pub fn invalid_param(param: &str, reason: &str) -> ToolError {
    ToolError::InvalidParam {
        message: format!("invalid parameter `{param}`: {reason}"),
        param: param.to_string(),
    }
}

/// Build an `unknown_instance` envelope listing valid labels.
#[must_use]
pub fn unknown_instance(label: &str, valid: Vec<String>) -> ToolError {
    ToolError::UnknownInstance {
        message: format!("unknown instance `{label}`"),
        valid,
    }
}

// ── DispatchError ─────────────────────────────────────────────────────────────

/// A structured MCP dispatch error with a stable `kind` tag.
///
/// Implements [`std::error::Error`] so it survives the `anyhow::Error` chain
/// and can be recovered via `anyhow::Error::downcast_ref` in `serve.rs`.
#[derive(Debug, Clone)]
pub struct DispatchError {
    /// Stable kind tag matching the MCP error vocabulary.
    pub kind: &'static str,
    /// Human-readable message.
    pub message: String,
    /// Optional list of valid values (for `unknown_action`, `unknown_instance`).
    pub valid: Option<Vec<String>>,
    /// Optional parameter name (for `missing_param`, `invalid_param`).
    pub param: Option<String>,
    /// Optional fuzzy hint.
    pub hint: Option<String>,
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for DispatchError {}

impl DispatchError {
    /// Unknown action — includes list of valid actions.
    #[must_use]
    pub fn unknown_action(service: &str, action: &str, valid: Vec<String>) -> Self {
        Self {
            kind: "unknown_action",
            message: format!("unknown action `{action}` for service `{service}`"),
            valid: Some(valid),
            param: None,
            hint: None,
        }
    }

    /// Required parameter missing.
    #[must_use]
    pub fn missing_param(param: &'static str) -> Self {
        Self {
            kind: "missing_param",
            message: format!("missing required parameter `{param}`"),
            valid: None,
            param: Some(param.to_string()),
            hint: None,
        }
    }

    /// Parameter present but wrong type or value.
    #[must_use]
    pub fn invalid_param(param: &'static str, reason: &str) -> Self {
        Self {
            kind: "invalid_param",
            message: format!("invalid parameter `{param}`: {reason}"),
            valid: None,
            param: Some(param.to_string()),
            hint: None,
        }
    }

    /// Unknown multi-instance label.
    #[must_use]
    pub fn unknown_instance(label: &str, valid: Vec<String>) -> Self {
        Self {
            kind: "unknown_instance",
            message: format!("unknown instance `{label}`"),
            valid: Some(valid),
            param: None,
            hint: None,
        }
    }

    /// Wrap an SDK/transport error preserving its kind tag.
    #[must_use]
    pub fn sdk(kind: &'static str, message: impl std::fmt::Display) -> Self {
        Self {
            kind,
            message: message.to_string(),
            valid: None,
            param: None,
            hint: None,
        }
    }
}

// `From<DispatchError> for anyhow::Error` is covered by anyhow's blanket
// `impl<E: StdError + Send + Sync + 'static> From<E> for anyhow::Error`.
// An explicit impl here would conflict — no explicit impl needed.

// ── From<ToolError> for DispatchError ────────────────────────────────────────
//
// Converts a `ToolError` to `DispatchError` without string round-tripping
// through `Display`. This keeps `extract_error_info` on path 1 (downcast)
// instead of path 2 (JSON re-parse), preserving all structured fields.

impl From<ToolError> for DispatchError {
    fn from(te: ToolError) -> Self {
        match te {
            ToolError::UnknownAction {
                message,
                valid,
                hint,
            } => Self {
                kind: "unknown_action",
                message,
                valid: Some(valid),
                param: None,
                hint,
            },
            ToolError::MissingParam { message, param } => Self {
                kind: "missing_param",
                message,
                valid: None,
                param: Some(param),
                hint: None,
            },
            ToolError::InvalidParam { message, param } => Self {
                kind: "invalid_param",
                message,
                valid: None,
                param: Some(param),
                hint: None,
            },
            ToolError::UnknownInstance { message, valid } => Self {
                kind: "unknown_instance",
                message,
                valid: Some(valid),
                param: None,
                hint: None,
            },
            ToolError::ConfirmationRequired { message } => Self {
                kind: "confirmation_required",
                message,
                valid: None,
                param: None,
                hint: None,
            },
            ToolError::Sdk { sdk_kind, message } => Self {
                kind: sdk_kind_to_static(&sdk_kind),
                message,
                valid: None,
                param: None,
                hint: None,
            },
        }
    }
}

/// Map a dynamic SDK kind string to a `&'static str` from the canonical vocabulary.
///
/// Must stay in sync with `serve::static_kind` and `ToolError::kind()`.
fn sdk_kind_to_static(s: &str) -> &'static str {
    match s {
        "unknown_action" => "unknown_action",
        "unknown_subaction" => "unknown_subaction",
        "missing_param" => "missing_param",
        "invalid_param" => "invalid_param",
        "unknown_instance" => "unknown_instance",
        "confirmation_required" => "confirmation_required",
        "auth_failed" => "auth_failed",
        "not_found" => "not_found",
        "rate_limited" => "rate_limited",
        "validation_failed" => "validation_failed",
        "network_error" => "network_error",
        "server_error" => "server_error",
        "decode_error" => "decode_error",
        _ => "internal_error",
    }
}
