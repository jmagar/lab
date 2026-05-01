use serde_json::Value;

use lab_apis::freshrss::FreshrssClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{
    action_schema, help_payload, optional_str, optional_u32, require_str, to_json,
};

use super::catalog::ACTIONS;
use super::client::require_client;

/// Dispatch an action using the scaffolded service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("freshrss", ACTIONS)),
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
    client: &FreshrssClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("freshrss", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "subscription.list" => to_json(client.subscriptions().await?),
        "tag.list" => to_json(client.tags().await?),
        "unread.counts" => to_json(client.unread_counts().await?),
        "stream.items" => {
            let n = optional_u32(&params, "n")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `n`".into(),
                param: "n".into(),
            })?;
            let continuation = optional_str(&params, "continuation")?;
            to_json(client.stream_items(n, continuation).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `freshrss`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
