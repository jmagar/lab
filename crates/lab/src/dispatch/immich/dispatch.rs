use serde_json::Value;

use lab_apis::immich::ImmichClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_u32, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;

/// Dispatch an action using the scaffolded service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("immich", ACTIONS)),
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
    client: &ImmichClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("immich", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.health" => {
            client.health().await?;
            Ok(Value::Null)
        }
        "server.info" => to_json(client.server_info().await?),
        "server.version" => to_json(client.server_version().await?),
        "user.me" => to_json(client.user_me().await?),
        "asset.search" => {
            let limit = optional_u32(&params, "limit")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `limit`".into(),
                param: "limit".into(),
            })?;
            let request = lab_apis::immich::types::AssetSearchRequest {
                query: params
                    .get("query")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                limit,
                page: optional_u32(&params, "page")?,
            };
            to_json(client.asset_search(request).await?)
        }
        "asset.get" => {
            let id = require_str(&params, "id")?;
            to_json(client.asset_get(id).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `immich`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
