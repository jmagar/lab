use lab_apis::gotify::GotifyClient;
use serde_json::Value;

use crate::dispatch::gotify::{catalog::ACTIONS, params};
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Dispatch against a pre-built client (avoids per-request env reads).
pub async fn dispatch_with_client(
    client: &GotifyClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Messages ─────────────────────────────────────────────────────────
        "message.send" => {
            let msg = params::send_message_from_params(&params_value)?;
            to_json(client.message_send(&msg).await?)
        }
        "message.list" => {
            let limit = params_value
                .get("limit")
                .and_then(Value::as_u64)
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            to_json(client.messages_list(limit).await?)
        }
        "message.delete" => {
            let id = params::message_id_from_params(&params_value)?;
            client.message_delete(id).await?;
            Ok(Value::Null)
        }
        "message.purge" => {
            client.messages_purge().await?;
            Ok(Value::Null)
        }
        // ── Applications ──────────────────────────────────────────────────────
        "app.list" => to_json(client.apps_list().await?),
        "app.create" => {
            let p = params::application_params_from_params(&params_value)?;
            to_json(client.app_create(&p).await?)
        }
        "app.delete" => {
            let id = params::application_id_from_params(&params_value)?;
            client.app_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Clients ───────────────────────────────────────────────────────────
        "client.list" => to_json(client.clients_list().await?),
        "client.create" => {
            let p = params::client_params_from_params(&params_value)?;
            to_json(client.client_create(&p).await?)
        }
        "client.delete" => {
            let id = params::client_id_from_params(&params_value)?;
            client.client_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Server ────────────────────────────────────────────────────────────
        "server.health" => to_json(client.server_health().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Top-level dispatch: handles built-ins then routes to `dispatch_with_client`.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("gotify", ACTIONS)),
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
    dispatch_with_client(&super::client::require_client()?, action, params_value).await
}
