use lab_apis::memos::MemosClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::memos::catalog::ACTIONS;
use crate::dispatch::memos::client;
use crate::dispatch::memos::params;

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
///
/// Called by the API handler which holds a client from `AppState`.
pub async fn dispatch_with_client(
    client: &MemosClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("memos", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        // ── Memos ─────────────────────────────────────────────────────────
        "memos.list" => {
            let p = params::list_params_from(&params_value)?;
            to_json(client.memos_list(&p).await?)
        }
        "memos.get" => {
            let name = params::require_name(&params_value)?;
            to_json(client.memo_get(name).await?)
        }
        "memos.create" => {
            let req = params::create_request_from(&params_value)?;
            to_json(client.memo_create(&req).await?)
        }
        "memos.update" => {
            let name = params::require_name(&params_value)?;
            let req = params::update_request_from(&params_value)?;
            to_json(client.memo_update(name, &req).await?)
        }
        "memos.delete" => {
            let name = params::require_name(&params_value)?;
            client.memo_delete(name).await?;
            Ok(Value::Null)
        }
        // ── Tags ──────────────────────────────────────────────────────────
        "tags.list" => to_json(client.tags_list().await?),
        // ── Workspace ─────────────────────────────────────────────────────
        "workspace.profile" => to_json(client.workspace_profile().await?),
        // ── User ──────────────────────────────────────────────────────────
        "user.me" => to_json(client.user_me().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `Memos` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API adapters.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("memos", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params_value, "action")?;
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
    dispatch_with_client(&client::require_client()?, action, params_value).await
}
