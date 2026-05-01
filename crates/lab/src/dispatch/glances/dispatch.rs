use serde_json::Value;

use lab_apis::glances::GlancesClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_u32, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;

/// Dispatch an action using the scaffolded service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("glances", ACTIONS)),
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
    client: &GlancesClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("glances", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.health" => {
            client.health().await?;
            Ok(Value::Null)
        }
        "system.info" => to_json(client.plugin("system").await?),
        "quicklook.summary" => to_json(client.plugin("quicklook").await?),
        "cpu.summary" => to_json(client.plugin("cpu").await?),
        "memory.summary" => to_json(client.plugin("mem").await?),
        "load.summary" => to_json(client.plugin("load").await?),
        "network.summary" => to_json(client.plugin("network").await?),
        "diskio.summary" => to_json(client.plugin("diskio").await?),
        "fs.summary" => to_json(client.plugin("fs").await?),
        "process.top" => {
            let limit = optional_u32(&params, "limit")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `limit`".into(),
                param: "limit".into(),
            })?;
            to_json(client.process_top(limit).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `glances`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
