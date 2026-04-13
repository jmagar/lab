use serde_json::Value;

use lab_apis::arcane::ArcaneClient;

use super::catalog::ACTIONS;
use super::params;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Dispatch against a pre-built Arcane client (avoids per-request env reads).
///
/// Called by the API handler with the client from `AppState`.
pub async fn dispatch_with_client(
    client: &ArcaneClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Health ────────────────────────────────────────────────────────────
        "health" => to_json(client.health().await?),

        // ── Environments ──────────────────────────────────────────────────────
        "environment.list" => to_json(client.environments_list().await?),
        "environment.get" => {
            let id = params::id_from_params(&params_value)?;
            to_json(client.environment_get(&id).await?)
        }

        // ── Containers ────────────────────────────────────────────────────────
        "container.list" => {
            let env_id = params::env_id_from_params(&params_value)?;
            to_json(client.containers_list(&env_id).await?)
        }
        "container.get" => {
            let env_id = params::env_id_from_params(&params_value)?;
            let container_id = params::container_id_from_params(&params_value)?;
            to_json(client.container_get(&env_id, &container_id).await?)
        }
        "container.start" => {
            let env_id = params::env_id_from_params(&params_value)?;
            let container_id = params::container_id_from_params(&params_value)?;
            to_json(client.container_start(&env_id, &container_id).await?)
        }
        "container.stop" => {
            let env_id = params::env_id_from_params(&params_value)?;
            let container_id = params::container_id_from_params(&params_value)?;
            to_json(client.container_stop(&env_id, &container_id).await?)
        }
        "container.restart" => {
            let env_id = params::env_id_from_params(&params_value)?;
            let container_id = params::container_id_from_params(&params_value)?;
            to_json(client.container_restart(&env_id, &container_id).await?)
        }
        "container.redeploy" => {
            let env_id = params::env_id_from_params(&params_value)?;
            let container_id = params::container_id_from_params(&params_value)?;
            to_json(client.container_redeploy(&env_id, &container_id).await?)
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Top-level dispatch: handles built-ins then routes to `dispatch_with_client`.
///
/// Called by MCP and CLI surfaces.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("arcane", ACTIONS)),
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
    let client = super::client::require_client()?;
    dispatch_with_client(&client, action, params_value).await
}
