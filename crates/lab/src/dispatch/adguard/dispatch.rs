use serde_json::Value;

use lab_apis::adguard::AdguardClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{
    action_schema, help_payload, optional_str, optional_u32, require_str, to_json,
};

use super::catalog::ACTIONS;
use super::client::require_client;

/// Dispatch an action using the scaffolded service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("adguard", ACTIONS)),
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
    client: &AdguardClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("adguard", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.status" => to_json(client.status().await?),
        "server.version" => to_json(client.version().await?),
        "stats.summary" => to_json(client.stats().await?),
        "querylog.search" => {
            let limit = optional_u32(&params, "limit")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `limit`".into(),
                param: "limit".into(),
            })?;
            to_json(
                client
                    .querylog(
                        limit,
                        optional_str(&params, "search")?,
                        optional_str(&params, "older_than")?,
                    )
                    .await?,
            )
        }
        "filtering.status" => to_json(client.filtering_status().await?),
        "filtering.check-host" => {
            let host = require_str(&params, "host")?;
            to_json(client.filtering_check_host(host).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `adguard`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
