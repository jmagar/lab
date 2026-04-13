use serde_json::Value;

use super::client::GotifyClients;
use crate::dispatch::error::ToolError;
use crate::dispatch::gotify::{catalog::ACTIONS, params};
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

fn require_app_client(
    clients: &GotifyClients,
) -> Result<&lab_apis::gotify::GotifyClient, ToolError> {
    clients
        .app()
        .ok_or_else(super::client::not_configured_error)
}

fn require_management_client(
    clients: &GotifyClients,
) -> Result<&lab_apis::gotify::GotifyClient, ToolError> {
    clients
        .client()
        .ok_or_else(super::client::not_configured_error)
}

/// Dispatch against pre-built Gotify clients (avoids per-request env reads).
pub async fn dispatch_with_client(
    clients: &GotifyClients,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Messages ─────────────────────────────────────────────────────────
        "message.send" => {
            let client = require_app_client(clients)?;
            let msg = params::send_message_from_params(&params_value)?;
            to_json(client.message_send(&msg).await?)
        }
        "message.list" => {
            let client = require_management_client(clients)?;
            let limit = crate::dispatch::helpers::optional_u32_max(&params_value, "limit", 200)?;
            to_json(client.messages_list(limit).await?)
        }
        "message.delete" => {
            let client = require_management_client(clients)?;
            let id = params::message_id_from_params(&params_value)?;
            client.message_delete(id).await?;
            Ok(Value::Null)
        }
        "message.purge" => {
            let client = require_management_client(clients)?;
            client.messages_purge().await?;
            Ok(Value::Null)
        }
        // ── Applications ──────────────────────────────────────────────────────
        "app.list" => to_json(require_management_client(clients)?.apps_list().await?),
        "app.create" => {
            let client = require_management_client(clients)?;
            let p = params::application_params_from_params(&params_value)?;
            to_json(client.app_create(&p).await?)
        }
        "app.delete" => {
            let client = require_management_client(clients)?;
            let id = params::application_id_from_params(&params_value)?;
            client.app_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Clients ───────────────────────────────────────────────────────────
        "client.list" => to_json(require_management_client(clients)?.clients_list().await?),
        "client.create" => {
            let client = require_management_client(clients)?;
            let p = params::client_params_from_params(&params_value)?;
            to_json(client.client_create(&p).await?)
        }
        "client.delete" => {
            let client = require_management_client(clients)?;
            let id = params::client_id_from_params(&params_value)?;
            client.client_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Server ────────────────────────────────────────────────────────────
        "server.health" => to_json(clients.health().server_health().await?),
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
    let clients =
        super::client::clients_from_env().ok_or_else(super::client::not_configured_error)?;
    dispatch_with_client(&clients, action, params_value).await
}
