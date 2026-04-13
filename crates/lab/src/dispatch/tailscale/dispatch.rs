use serde_json::Value;

use lab_apis::tailscale::TailscaleClient;

use super::catalog::ACTIONS;
use super::params;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Dispatch against a pre-built `TailscaleClient` (avoids per-request env reads).
///
/// Called by the API handler with the client from `AppState`.
pub async fn dispatch_with_client(
    client: &TailscaleClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Devices ──────────────────────────────────────────────────────────
        "device.list" => to_json(client.devices_list().await?),
        "device.get" => {
            let device_id = params::device_id_from_params(&params_value)?;
            to_json(client.device_get(&device_id).await?)
        }
        "device.delete" => {
            let device_id = params::device_id_from_params(&params_value)?;
            client.device_delete(&device_id).await?;
            Ok(Value::Null)
        }
        "device.authorize" => {
            let device_id = params::device_id_from_params(&params_value)?;
            let authorized = params::authorized_from_params(&params_value)?;
            client.device_authorize(&device_id, authorized).await?;
            Ok(Value::Null)
        }
        // ── Auth Keys ────────────────────────────────────────────────────────
        "key.list" => to_json(client.keys_list().await?),
        "key.get" => {
            let key_id = params::key_id_from_params(&params_value)?;
            to_json(client.key_get(&key_id).await?)
        }
        "key.delete" => {
            let key_id = params::key_id_from_params(&params_value)?;
            client.key_delete(&key_id).await?;
            Ok(Value::Null)
        }
        // ── DNS ──────────────────────────────────────────────────────────────
        "dns.nameservers" => to_json(client.dns_nameservers().await?),
        "dns.search_paths" => to_json(client.dns_search_paths().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Top-level dispatch: handles built-ins then routes to `dispatch_with_client`.
///
/// Used by MCP and CLI surfaces.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("tailscale", ACTIONS)),
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
