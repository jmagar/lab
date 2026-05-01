use serde_json::Value;

use lab_apis::navidrome::NavidromeClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_u32, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;

/// Dispatch an action using the scaffolded service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("navidrome", ACTIONS)),
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
    client: &NavidromeClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("navidrome", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.ping" => to_json(client.ping().await?),
        "artist.list" => to_json(client.artists().await?),
        "album.list" => {
            let list_type = params
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("newest");
            let size = optional_u32(&params, "size")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `size`".into(),
                param: "size".into(),
            })?;
            let offset = optional_u32(&params, "offset")?.unwrap_or(0);
            to_json(client.album_list(list_type, size, offset).await?)
        }
        "album.get" => {
            let id = require_str(&params, "id")?;
            to_json(client.album_get(id).await?)
        }
        "search.query" => {
            let query = require_str(&params, "query")?;
            let size = optional_u32(&params, "size")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `size`".into(),
                param: "size".into(),
            })?;
            let offset = optional_u32(&params, "offset")?.unwrap_or(0);
            to_json(client.search(query, size, offset).await?)
        }
        "playlist.list" => to_json(client.playlists().await?),
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `navidrome`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
