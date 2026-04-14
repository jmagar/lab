use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::client::require_client;
use super::{
    calendar, catalog::actions, commands, config, customformat, history, movies, queue, system,
    wanted,
};

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
        "history.list" | "history.movie" | "history.failed-retry" | "blocklist.list"
        | "blocklist.delete" => history::dispatch_with_client(client, action, params).await,
        a if a.starts_with("wanted.") => wanted::dispatch_with_client(client, action, params).await,
        "customformat.list" => customformat::dispatch_with_client(client, action, params).await,
        "release.search"
        | "release.grab"
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
