use lab_apis::tautulli::TautulliClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_i64, require_str, to_json};
use crate::dispatch::tautulli::{catalog::ACTIONS, client, params};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &TautulliClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Activity ──────────────────────────────────────────────────────────
        "activity.list" => to_json(client.get_activity().await?),
        "activity.stream" => {
            let session_key = require_str(&params_value, "session_key")?;
            to_json(client.get_stream_data(session_key).await?)
        }
        // ── History ───────────────────────────────────────────────────────────
        "history.list" => {
            let query = params::history_query_from_params(&params_value)?;
            to_json(client.get_history(&query).await?)
        }
        // ── Users ─────────────────────────────────────────────────────────────
        "users.list" => to_json(client.get_users().await?),
        "users.get" => {
            let user_id = params::require_user_id(&params_value)?;
            to_json(client.get_user(user_id).await?)
        }
        "users.watch_time" => {
            let user_id = params::require_user_id(&params_value)?;
            to_json(client.get_user_watch_time_stats(user_id).await?)
        }
        "users.player_stats" => {
            let user_id = params::require_user_id(&params_value)?;
            to_json(client.get_user_player_stats(user_id).await?)
        }
        // ── Libraries ─────────────────────────────────────────────────────────
        "libraries.list" => to_json(client.get_libraries().await?),
        "libraries.get" => {
            let section_id = params::require_section_id(&params_value)?;
            to_json(client.get_library(section_id).await?)
        }
        "libraries.media_info" => {
            let section_id = params::require_section_id(&params_value)?;
            to_json(client.get_library_media_info(section_id).await?)
        }
        // ── Statistics ────────────────────────────────────────────────────────
        "stats.home" => {
            let (time_range, stats_count) = params::home_stats_params(&params_value)?;
            to_json(client.get_home_stats(time_range, stats_count).await?)
        }
        "stats.plays_by_date" => {
            let (time_range, y_axis) = params::plays_by_date_params(&params_value)?;
            to_json(
                client
                    .get_plays_by_date(time_range, y_axis.as_deref())
                    .await?,
            )
        }
        // ── Media ─────────────────────────────────────────────────────────────
        "media.recently-added" => {
            let count = params::optional_count(&params_value)?;
            let section_id = optional_str(&params_value, "section_id")?.map(str::to_owned);
            to_json(
                client
                    .get_recently_added(count, section_id.as_deref())
                    .await?,
            )
        }
        "media.metadata" => {
            let rating_key = params::require_rating_key(&params_value)?;
            to_json(client.get_metadata(rating_key).await?)
        }
        "media.children" => {
            let rating_key = params::require_rating_key(&params_value)?;
            to_json(client.get_children_metadata(rating_key).await?)
        }
        "media.export-metadata" => {
            let rating_key = params::require_rating_key(&params_value)?;
            let media_type = require_str(&params_value, "media_type")?;
            to_json(client.export_metadata(rating_key, media_type).await?)
        }
        // ── User stats ────────────────────────────────────────────────────────
        "user.item-stats" => {
            let rating_key = params::require_rating_key(&params_value)?;
            let media_type = optional_str(&params_value, "media_type")?.map(str::to_owned);
            to_json(
                client
                    .get_item_user_stats(rating_key, media_type.as_deref())
                    .await?,
            )
        }
        "user.delete-history" => {
            let user_id = require_i64(&params_value, "user_id")?;
            to_json(client.delete_all_user_history(user_id).await?)
        }
        // ── Play analytics ────────────────────────────────────────────────────
        "plays.by-day" => {
            let time_range = params::optional_time_range(&params_value)?;
            to_json(client.get_plays_by_dayofweek(time_range).await?)
        }
        "plays.by-hour" => {
            let time_range = params::optional_time_range(&params_value)?;
            to_json(client.get_plays_by_hourofday(time_range).await?)
        }
        "plays.by-stream-type" => {
            let time_range = params::optional_time_range(&params_value)?;
            to_json(client.get_plays_by_stream_type(time_range).await?)
        }
        "plays.by-month" => {
            let time_range = params::optional_time_range(&params_value)?;
            to_json(client.get_plays_per_month(time_range).await?)
        }
        // ── Server ────────────────────────────────────────────────────────────
        "server.pms-update" => to_json(client.get_pms_update().await?),
        // ── System ────────────────────────────────────────────────────────────
        "system.info" => to_json(client.get_server_info().await?),
        "system.settings" => to_json(client.get_settings().await?),

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `Tautulli` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("tautulli", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}'"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&client::require_client()?, action, params_value).await
}
