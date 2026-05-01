use serde_json::Value;

use lab_apis::uptime_kuma::UptimeKumaClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("uptime-kuma", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => Err(unknown_action(action)),
        _ => dispatch_with_client(&require_client()?, action, params).await,
    }
}

/// Dispatch an action using a prebuilt client.
///
/// `help` and `schema` are intercepted by [`dispatch`] and never reach here.
pub async fn dispatch_with_client(
    client: &UptimeKumaClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("uptime-kuma", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "contract.status" => to_json(client.contract_status()),
        "server.health" => {
            client.health().await?;
            Ok(Value::Null)
        }
        "monitor.get" => {
            let _id = require_str(&params, "id")?;
            to_json(client.unsupported_socket_read(action).await?)
        }
        "monitor.list" | "heartbeat.list" | "status.summary" | "notification.list" => {
            to_json(client.unsupported_socket_read(action).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `uptime-kuma`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
