//! Command resource dispatch: refresh, search, get.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::types::{CommandId, MovieId};
use serde_json::Value;

use super::client::require_client;
use super::params::{require_i64, to_json};
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "command.refresh",
        description: "Refresh metadata for one movie or all movies",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "movie_id",
            ty: "i64",
            required: false,
            description: "Movie ID to refresh (omit to refresh all)",
        }],
    },
    ActionSpec {
        name: "command.search",
        description: "Trigger a file search for specified movies",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "movie_ids",
            ty: "i64[]",
            required: true,
            description: "Array of Radarr movie IDs to search",
        }],
    },
    ActionSpec {
        name: "command.get",
        description: "Get the status of a previously issued command",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Command ID",
        }],
    },
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "command.refresh" => {
            let movie_id = params.get("movie_id").and_then(Value::as_i64).map(MovieId);
            let cmd = client.command_refresh_movie(movie_id).await?;
            to_json(cmd)
        }
        "command.search" => {
            let ids: Vec<MovieId> = params
                .get("movie_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `movie_ids`".to_string(),
                    param: "movie_ids".to_string(),
                })?
                .iter()
                .filter_map(Value::as_i64)
                .map(MovieId)
                .collect();
            let cmd = client.command_movies_search(&ids).await?;
            to_json(cmd)
        }
        "command.get" => {
            let id = CommandId(require_i64(&params, "id")?);
            let cmd = client.command_get(id).await?;
            to_json(cmd)
        }
        _ => unreachable!(),
    }
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    dispatch_with_client(&require_client()?, action, params).await
}
