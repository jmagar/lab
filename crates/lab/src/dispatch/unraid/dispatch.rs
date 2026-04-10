//! Top-level action routing for the `unraid` service.

use lab_apis::unraid::UnraidClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::unraid::{catalog::ACTIONS, client::require_client, params::require_id};

/// Dispatch using a pre-built client (avoids per-request env reads).
///
/// Called by the API handler which holds a client in `AppState`.
pub async fn dispatch_with_client(
    client: &UnraidClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "system.info" => to_json(client.system_info().await?),
        "system.metrics" => to_json(client.system_metrics().await?),
        "system.array" => to_json(client.system_array().await?),
        "system.online" => {
            let online = client.system_online().await?;
            Ok(serde_json::json!({ "online": online }))
        }
        "docker.list" => to_json(client.docker_list().await?),
        "docker.start" => {
            let id = require_id(&params)?;
            client.docker_start(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "docker.stop" => {
            let id = require_id(&params)?;
            client.docker_stop(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "docker.restart" => {
            let id = require_id(&params)?;
            client.docker_restart(&id).await?;
            Ok(serde_json::json!({ "ok": true, "id": id }))
        }
        "disk.list" => to_json(client.disk_list().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `unraid` service.
///
/// Handles the built-in `help` and `schema` actions before delegating to
/// `dispatch_with_client`.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("unraid", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params, "action")?;
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
    dispatch_with_client(&require_client()?, action, params).await
}
