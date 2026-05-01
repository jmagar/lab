use lab_apis::jellyfin::JellyfinClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_str, to_json};

use super::catalog::ACTIONS;
use super::params;

/// Dispatch one call against the Jellyfin service.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("jellyfin", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ => {}
    }

    if !ACTIONS.iter().any(|spec| spec.name == action) {
        return Err(unknown_action(action));
    }

    let instance = optional_str(&params, "instance")?.map(str::to_owned);
    let mut params = params;
    if let Value::Object(ref mut map) = params {
        map.remove("instance");
    }

    let client = super::client::client_from_instance(instance.as_deref())?;
    dispatch_with_client(&client, action, params).await
}

/// Dispatch one Jellyfin call with an explicit client.
pub async fn dispatch_with_client(
    client: &JellyfinClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "system.ping" => to_json(client.ping().await?),
        "system.info" => to_json(client.system_info().await?),
        "system.public_info" => to_json(client.public_system_info().await?),
        "users.list" => to_json(client.users().await?),
        "users.me" => to_json(client.current_user().await?),
        "libraries.list" => to_json(client.libraries().await?),
        "items.search" => {
            let query = params::items_query(&params)?;
            to_json(client.items(&query).await?)
        }
        "items.get" => {
            let item_id = params::item_id(&params)?;
            to_json(client.item(item_id).await?)
        }
        "items.counts" => to_json(client.item_counts().await?),
        "sessions.list" => to_json(client.sessions().await?),
        "plugins.list" => to_json(client.plugins().await?),
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `jellyfin`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
