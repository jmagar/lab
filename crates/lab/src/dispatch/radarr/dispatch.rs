use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::client::require_client;
use super::{calendar, catalog::actions, commands, config, history, movies, queue, system};

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        a if a.starts_with("system.") => system::dispatch_with_client(client, action, params).await,
        a if a.starts_with("movie.") => movies::dispatch_with_client(client, action, params).await,
        a if a.starts_with("queue.") => queue::dispatch_with_client(client, action, params).await,
        "calendar.list" => calendar::dispatch_with_client(client, action, params).await,
        a if a.starts_with("command.") => {
            commands::dispatch_with_client(client, action, params).await
        }
        "history.list" | "blocklist.list" => {
            history::dispatch_with_client(client, action, params).await
        }
        "release.search"
        | "indexer.list"
        | "indexer.test"
        | "quality-profile.list"
        | "quality-definition.list"
        | "root-folder.list"
        | "tag.list"
        | "tag.detail-list"
        | "download-client.list"
        | "download-client.test"
        | "remote-path-mapping.list"
        | "config.host"
        | "config.naming"
        | "config.ui"
        | "notification.list"
        | "notification.test"
        | "import-list.list"
        | "import-list.exclusion-list"
        | "language.list"
        | "metadata.list"
        | "filesystem.list" => config::dispatch_with_client(client, action, params).await,
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `radarr`"),
            valid: actions().iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("radarr", actions())),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            action_schema(actions(), action_name)
        }
        _ => dispatch_with_client(&require_client()?, action, params).await,
    }
}
