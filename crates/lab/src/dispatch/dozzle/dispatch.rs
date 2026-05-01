use serde_json::Value;

use lab_apis::dozzle::DozzleClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{limits_from_params, log_fetch_request_from_params};

/// Dispatch an action using the service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("dozzle", ACTIONS)),
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
    client: &DozzleClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("dozzle", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.health" => to_json(client.health().await?),
        "server.version" => to_json(client.version().await?),
        "containers.list" => {
            let limits = limits_from_params(&params)?;
            to_json(client.containers_list(limits).await?)
        }
        "logs.fetch" => {
            let request = log_fetch_request_from_params(&params)?;
            to_json(client.logs_fetch(&request).await?)
        }
        "logs.fetch-plain" => {
            let request = log_fetch_request_from_params(&params)?;
            to_json(client.logs_fetch_plain(&request).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `dozzle`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
