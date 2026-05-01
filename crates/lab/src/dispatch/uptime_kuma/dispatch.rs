use serde_json::Value;

use lab_apis::uptime_kuma::UptimeKumaClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{hours, monitor_id, optional_monitor_id};

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
        "monitor.list" => to_json(client.monitor_list().await?),
        "monitor.get" => to_json(client.monitor_get(monitor_id(&params, "id")?).await?),
        "heartbeat.list" => to_json(
            client
                .heartbeat_list(optional_monitor_id(&params)?, hours(&params)?)
                .await?,
        ),
        "status.summary" => to_json(client.status_summary().await?),
        "notification.list" => to_json(client.notification_list().await?),
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
