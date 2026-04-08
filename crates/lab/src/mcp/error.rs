//! Helper constructors for [`ToolError`] envelopes. Dispatchers should
//! prefer these over building variants inline — keeps envelope shape
//! consistent across services.

use crate::mcp::envelope::ToolError;

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
