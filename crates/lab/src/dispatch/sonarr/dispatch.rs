//! Action routing for the `Sonarr` dispatch layer.

use lab_apis::sonarr::SonarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{
    add_series_from_params, calendar_query_from_params, episode_ids_from_params,
    history_query_from_params, optional_u32, queue_query_from_params, require_i64, require_str,
};

/// Dispatch using a pre-built client (avoids per-request env reads and
/// client construction). Called by the API handler.
pub async fn dispatch_with_client(
    client: &SonarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Series ────────────────────────────────────────────────────────────
        "series.list" => to_json(client.series_list().await?),
        "series.get" => {
            let id = require_i64(&params, "id")?;
            to_json(client.series_get(id).await?)
        }
        "series.lookup" => {
            let query = require_str(&params, "query")?;
            to_json(client.series_lookup(query).await?)
        }
        "series.add" => {
            let req = add_series_from_params(&params)?;
            to_json(client.series_add(&req).await?)
        }
        "series.delete" => {
            let id = require_i64(&params, "id")?;
            let delete_files = params
                .get("delete_files")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            client.series_delete(id, delete_files).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        // ── Episodes ──────────────────────────────────────────────────────────
        "episode.list" => {
            let series_id = require_i64(&params, "series_id")?;
            to_json(client.episode_list(series_id).await?)
        }
        "episode.get" => {
            let id = require_i64(&params, "id")?;
            to_json(client.episode_get(id).await?)
        }
        // ── Queue ─────────────────────────────────────────────────────────────
        "queue.list" => {
            let query = queue_query_from_params(&params)?;
            to_json(client.queue_list(&query).await?)
        }
        "queue.delete" => {
            let id = require_i64(&params, "id")?;
            let blocklist = params
                .get("blocklist")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            client.queue_delete(id, blocklist).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        // ── History ───────────────────────────────────────────────────────────
        "history.list" => {
            let query = history_query_from_params(&params)?;
            to_json(client.history_list(&query).await?)
        }
        // ── Wanted ────────────────────────────────────────────────────────────
        "wanted.list" => {
            let page = optional_u32(&params, "page")?;
            let page_size = optional_u32(&params, "page_size")?;
            to_json(client.wanted_missing(page, page_size).await?)
        }
        // ── Calendar ──────────────────────────────────────────────────────────
        "calendar.list" => {
            let query = calendar_query_from_params(&params)?;
            to_json(client.calendar_list(&query).await?)
        }
        // ── Health & System ───────────────────────────────────────────────────
        "health" => to_json(client.health().await?),
        "system.status" => to_json(client.system_status().await?),
        // ── Tags ──────────────────────────────────────────────────────────────
        "tag.list" => to_json(client.tag_list().await?),
        "tag.create" => {
            let label = require_str(&params, "label")?;
            to_json(client.tag_create(label).await?)
        }
        "tag.delete" => {
            let id = require_i64(&params, "id")?;
            client.tag_delete(id).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        // ── Root Folders ──────────────────────────────────────────────────────
        "rootfolder.list" => to_json(client.rootfolder_list().await?),
        // ── Quality Profiles ──────────────────────────────────────────────────
        "qualityprofile.list" => to_json(client.qualityprofile_list().await?),
        // ── Language Profiles ─────────────────────────────────────────────────
        "languageprofile.list" => to_json(client.languageprofile_list().await?),
        // ── Series Edit ───────────────────────────────────────────────────────
        "series.edit" => {
            let id = require_i64(&params, "id")?;
            let body = params
                .get("body")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam { param: "body".to_string(), message: "parameter `body` is required".to_string() })?;
            to_json(client.series_edit(id, &body).await?)
        }
        // ── Episode Monitor ───────────────────────────────────────────────────
        "episode.monitor" => {
            let episode_ids = episode_ids_from_params(&params)?;
            let monitored = params
                .get("monitored")
                .and_then(Value::as_bool)
                .ok_or_else(|| ToolError::MissingParam { param: "monitored".to_string(), message: "parameter `monitored` is required".to_string() })?;
            to_json(client.episode_monitor(&episode_ids, monitored).await?)
        }
        // ── Wanted Cutoff ─────────────────────────────────────────────────────
        "wanted.cutoff" => {
            let page = optional_u32(&params, "page")?;
            let page_size = optional_u32(&params, "page_size")?;
            to_json(client.wanted_cutoff(page, page_size).await?)
        }
        // ── Releases ──────────────────────────────────────────────────────────
        "release.search" => {
            let series_id = params.get("series_id").and_then(Value::as_i64);
            let season_number = params.get("season_number").and_then(Value::as_i64).map(|n| n as i32);
            to_json(client.release_search(series_id, season_number).await?)
        }
        "release.grab" => {
            let guid = require_str(&params, "guid")?;
            let body = serde_json::json!({ "guid": guid });
            to_json(client.release_grab(&body).await?)
        }
        // ── History Series / Failed Retry ─────────────────────────────────────
        "history.series" => {
            let series_id = require_i64(&params, "series_id")?;
            to_json(client.history_series(series_id).await?)
        }
        "history.failed-retry" => {
            let id = require_i64(&params, "id")?;
            client.history_failed_retry(id).await?;
            Ok(serde_json::json!({ "retried": true }))
        }
        // ── Blocklist ─────────────────────────────────────────────────────────
        "blocklist.list" => to_json(client.blocklist_list().await?),
        "blocklist.delete" => {
            let id = require_i64(&params, "id")?;
            client.blocklist_delete(id).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        // ── Episode File ──────────────────────────────────────────────────────
        "episodefile.delete" => {
            let id = require_i64(&params, "id")?;
            client.episodefile_delete(id).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        // ── System ────────────────────────────────────────────────────────────
        "system.restart" => {
            client.system_restart().await?;
            Ok(serde_json::json!({ "restarted": true }))
        }
        "system.backup" => to_json(client.system_backup().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `Sonarr` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("sonarr", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
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
    dispatch_with_client(&require_client()?, action, params).await
}
