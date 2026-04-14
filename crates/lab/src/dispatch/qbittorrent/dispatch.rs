use lab_apis::qbittorrent::QbittorrentClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{
    optional_bool, optional_i64, optional_str, optional_u32, require_f64, require_i32, require_i64,
    require_str, to_json,
};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
#[allow(clippy::too_many_lines)]
pub async fn dispatch_with_client(
    client: &QbittorrentClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "transfer.info" => to_json(client.transfer_info().await?),

        "transfer.download.limit" => {
            let limit = require_i64(&params, "limit")?;
            client.set_download_limit(limit).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "transfer.upload.limit" => {
            let limit = require_i64(&params, "limit")?;
            client.set_upload_limit(limit).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.list" => {
            let filter = optional_str(&params, "filter")?;
            let category = optional_str(&params, "category")?;
            let limit = optional_u32(&params, "limit")?;
            to_json(client.list_torrents(filter, category, limit).await?)
        }

        "torrent.properties" => {
            let hash = require_str(&params, "hash")?;
            to_json(client.torrent_properties(hash).await?)
        }

        "torrent.trackers" => {
            let hash = require_str(&params, "hash")?;
            to_json(client.torrent_trackers(hash).await?)
        }

        "torrent.pause" => {
            let hashes = require_str(&params, "hashes")?;
            client.pause_torrents(hashes).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.resume" => {
            let hashes = require_str(&params, "hashes")?;
            client.resume_torrents(hashes).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.delete" => {
            let hashes = require_str(&params, "hashes")?;
            let delete_files = optional_bool(&params, "delete_files")?.unwrap_or(false);
            client.delete_torrents(hashes, delete_files).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.recheck" => {
            let hashes = require_str(&params, "hashes")?;
            client.recheck_torrents(hashes).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.category.set" => {
            let hashes = require_str(&params, "hashes")?;
            let category = require_str(&params, "category")?;
            client.set_category(hashes, category).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.download.limit" => {
            let hashes = require_str(&params, "hashes")?;
            let limit = require_i64(&params, "limit")?;
            client.set_torrent_download_limit(hashes, limit).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.upload.limit" => {
            let hashes = require_str(&params, "hashes")?;
            let limit = require_i64(&params, "limit")?;
            client.set_torrent_upload_limit(hashes, limit).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "category.list" => {
            let map = client.categories().await?;
            let list: Vec<_> = map.into_values().collect();
            to_json(list)
        }

        "app.version" => {
            let v = client.version().await?;
            Ok(serde_json::json!({ "version": v }))
        }

        "app.preferences" => to_json(client.preferences().await?),

        "log.list" => {
            let last_known_id = optional_i64(&params, "last_known_id")?;
            to_json(client.log(last_known_id).await?)
        }

        "torrent.add" => {
            let urls = require_str(&params, "urls")?;
            let savepath = optional_str(&params, "savepath")?;
            let category = optional_str(&params, "category")?;
            let tags = optional_str(&params, "tags")?;
            client.add_torrent(urls, savepath, category, tags).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "transfer.toggle-speed-limits" => {
            client.toggle_speed_limits().await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.files" => {
            let hash = require_str(&params, "hash")?;
            to_json(client.torrent_files(hash).await?)
        }

        "torrent.set-file-prio" => {
            let hash = require_str(&params, "hash")?;
            let id = require_str(&params, "id")?;
            let priority = require_i32(&params, "priority")?;
            client.set_file_priority(hash, id, priority).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.set-location" => {
            let hashes = require_str(&params, "hashes")?;
            let location = require_str(&params, "location")?;
            client.set_location(hashes, location).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.add-tags" => {
            let hashes = require_str(&params, "hashes")?;
            let tags = require_str(&params, "tags")?;
            client.add_tags(hashes, tags).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.remove-tags" => {
            let hashes = require_str(&params, "hashes")?;
            let tags = require_str(&params, "tags")?;
            client.remove_tags(hashes, tags).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.reannounce" => {
            let hashes = require_str(&params, "hashes")?;
            client.reannounce(hashes).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "torrent.set-share-limits" => {
            let hashes = require_str(&params, "hashes")?;
            let ratio_limit = require_f64(&params, "ratio_limit")?;
            let seeding_time_limit = require_i64(&params, "seeding_time_limit")?;
            let inactive_seeding_time_limit =
                require_i64(&params, "inactive_seeding_time_limit")?;
            client
                .set_share_limits(hashes, ratio_limit, seeding_time_limit, inactive_seeding_time_limit)
                .await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "category.create" => {
            let category = require_str(&params, "category")?;
            let savepath = optional_str(&params, "savepath")?;
            client.create_category(category, savepath).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "category.edit" => {
            let category = require_str(&params, "category")?;
            let savepath = require_str(&params, "savepath")?;
            client.edit_category(category, savepath).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "sync.maindata" => {
            let rid = optional_i64(&params, "rid")?.unwrap_or(0);
            if rid == 0 {
                tracing::warn!(
                    action = "sync.maindata",
                    rid = 0,
                    "rid=0 triggers full dump; pass returned rid on subsequent calls"
                );
            }
            to_json(client.sync_maindata(rid).await?)
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `qbittorrent`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch from CLI/MCP — resolves client from env then delegates.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("qbittorrent", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action `{action}` for service `qbittorrent`"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&require_client().await?, action, params).await
}
