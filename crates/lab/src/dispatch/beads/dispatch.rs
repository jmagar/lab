//! Top-level dispatch routing for the beads service.
//!
//! `dispatch` is what MCP and CLI call. `dispatch_with_client` is what
//! the API handler calls with a pre-built client from `AppState`.

use lab_apis::beads::{BeadsClient, BeadsError};
use serde_json::Value;

use crate::dispatch::beads::catalog::ACTIONS;
use crate::dispatch::beads::client::unavailable_error;
use crate::dispatch::beads::params;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Map a `BeadsError` to a `ToolError`.
///
/// Always uses `redacted_message()` rather than `Display` to avoid leaking
/// MySQL wire-protocol detail (table names, constraint names, hostnames)
/// through MCP / HTTP envelopes. Full detail is logged at WARN/ERROR.
fn beads_error_to_tool(err: BeadsError) -> ToolError {
    let kind = err.kind();
    let redacted = err.redacted_message();
    // Server-side log carries full Display detail.
    match kind {
        "internal_error" => tracing::error!(error = %err, "beads internal error"),
        _ => tracing::warn!(error = %err, kind, "beads error"),
    }
    ToolError::Sdk {
        sdk_kind: kind.to_string(),
        message: redacted,
    }
}

/// Dispatch with an optional pre-built client.
///
/// Used by the API handler so the help/schema branch is collapsed into a
/// single entry point: built-in actions never need a configured client; any
/// other action requires one or returns `beads_unavailable`.
pub async fn dispatch_with_optional_client(
    client: Option<&BeadsClient>,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("beads", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ => {}
    }
    let Some(client) = client else {
        return Err(unavailable_error());
    };
    dispatch_with_client(client, action, params_value).await
}

/// Dispatch using a pre-built client (avoids per-request env reads and
/// pool construction).
pub async fn dispatch_with_client(
    client: &BeadsClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("beads", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "issue.list" => {
            let p = params::issue_list_params(&params_value)?;
            let issues = client.list_issues(&p).await.map_err(beads_error_to_tool)?;
            to_json(serde_json::json!({ "issues": issues }))
        }
        "issue.get" => {
            let id = params::require_id(&params_value)?;
            let issue = client.get_issue(id).await.map_err(beads_error_to_tool)?;
            to_json(issue)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch a beads action without a pre-built client.
///
/// Falls back to constructing a fresh client from environment / port file.
/// Returns `beads_unavailable` if the Dolt port cannot be discovered.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("beads", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}'"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }

    let client = crate::dispatch::beads::client::client_from_env().ok_or_else(unavailable_error)?;
    dispatch_with_client(&client, action, params_value).await
}
