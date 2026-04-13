use lab_apis::sabnzbd::SabnzbdClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{opt_u32, require_str, require_u64, to_json};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &SabnzbdClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "server.version" => {
            let version = client.version().await?;
            Ok(serde_json::json!({ "version": version }))
        }
        "queue.list" => to_json(client.queue().await?),
        "queue.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = client.queue_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }
        "history.list" => {
            let limit = opt_u32(&params, "limit")?;
            to_json(client.history(limit).await?)
        }
        "history.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = client.history_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }
        "history.purge" => {
            let ok = client.history_purge().await?;
            Ok(serde_json::json!({ "purged": ok }))
        }
        "server.stats" => to_json(client.server_stats().await?),
        "server.warnings" => to_json(client.warnings().await?),
        "queue.pause" => {
            let ok = client.pause().await?;
            Ok(serde_json::json!({ "paused": ok }))
        }
        "queue.resume" => {
            let ok = client.resume().await?;
            Ok(serde_json::json!({ "resumed": ok }))
        }
        "queue.speed.limit" => {
            let kbps = require_u64(&params, "kbps")?;
            let ok = client.set_speed_limit(kbps).await?;
            Ok(serde_json::json!({ "speed_limit_set": ok }))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `sabnzbd`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("sabnzbd", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action `{action}` for service `sabnzbd`"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&require_client()?, action, params).await
}
