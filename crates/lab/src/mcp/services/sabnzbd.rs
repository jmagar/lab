//! MCP dispatch for the SABnzbd tool.

use serde_json::Value;

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::sabnzbd::SabnzbdClient;

use crate::services::error::ToolError;

fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Action catalog for the sabnzbd tool.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "version",
        description: "Return SABnzbd version string",
        destructive: false,
        returns: "string",
        params: &[],
    },
    ActionSpec {
        name: "queue.list",
        description: "List items in the download queue with status and progress",
        destructive: false,
        returns: "QueueResponse",
        params: &[],
    },
    ActionSpec {
        name: "queue.delete",
        description: "Delete an item from the download queue",
        destructive: true,
        returns: "bool",
        params: &[ParamSpec {
            name: "nzo_id",
            ty: "string",
            required: true,
            description: "NZO ID of the queue item to delete",
        }],
    },
    ActionSpec {
        name: "history.list",
        description: "List download history",
        destructive: false,
        returns: "HistoryResponse",
        params: &[ParamSpec {
            name: "limit",
            ty: "integer",
            required: false,
            description: "Max number of history items to return",
        }],
    },
    ActionSpec {
        name: "history.delete",
        description: "Delete a single history item",
        destructive: true,
        returns: "bool",
        params: &[ParamSpec {
            name: "nzo_id",
            ty: "string",
            required: true,
            description: "NZO ID of the history item to delete",
        }],
    },
    ActionSpec {
        name: "history.purge",
        description: "Purge all completed history items",
        destructive: true,
        returns: "bool",
        params: &[],
    },
    ActionSpec {
        name: "server-stats",
        description: "Return download statistics (total/month/week/day)",
        destructive: false,
        returns: "ServerStats",
        params: &[],
    },
    ActionSpec {
        name: "warnings",
        description: "List SABnzbd warnings",
        destructive: false,
        returns: "string[]",
        params: &[],
    },
    ActionSpec {
        name: "pause",
        description: "Pause the download queue",
        destructive: false,
        returns: "bool",
        params: &[],
    },
    ActionSpec {
        name: "resume",
        description: "Resume the download queue",
        destructive: false,
        returns: "bool",
        params: &[],
    },
    ActionSpec {
        name: "speed-limit",
        description: "Set the download speed limit in KB/s (0 = unlimited)",
        destructive: false,
        returns: "bool",
        params: &[ParamSpec {
            name: "kbps",
            ty: "integer",
            required: true,
            description: "Speed limit in KB/s (0 to remove limit)",
        }],
    },
];

/// Build a SABnzbd client from the default-instance env vars.
fn client_from_env() -> Option<SabnzbdClient> {
    let url = std::env::var("SABNZBD_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let key = std::env::var("SABNZBD_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
    SabnzbdClient::new(&url, key).ok()
}

fn require_client() -> Result<SabnzbdClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "SABNZBD_URL or SABNZBD_API_KEY not configured".to_string(),
    })
}

fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Dispatch one MCP call against the sabnzbd tool.
///
/// # Errors
/// Returns a `ToolError` on unknown actions, missing params, or SDK failures.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "sabnzbd",
            "actions": ACTIONS.iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "returns": a.returns,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),

        "version" => {
            let version = require_client()?.version().await?;
            Ok(serde_json::json!({ "version": version }))
        }

        "queue.list" => {
            let resp = require_client()?.queue().await?;
            to_json(resp)
        }

        "queue.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = require_client()?.queue_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }

        "history.list" => {
            let limit = params
                .get("limit")
                .and_then(Value::as_u64)
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let resp = require_client()?.history(limit).await?;
            to_json(resp)
        }

        "history.delete" => {
            let nzo_id = require_str(&params, "nzo_id")?;
            let ok = require_client()?.history_delete(nzo_id).await?;
            Ok(serde_json::json!({ "deleted": ok }))
        }

        "history.purge" => {
            let ok = require_client()?.history_purge().await?;
            Ok(serde_json::json!({ "purged": ok }))
        }

        "server-stats" => {
            let stats = require_client()?.server_stats().await?;
            to_json(stats)
        }

        "warnings" => {
            let warnings = require_client()?.warnings().await?;
            to_json(warnings)
        }

        "pause" => {
            let ok = require_client()?.pause().await?;
            Ok(serde_json::json!({ "paused": ok }))
        }

        "resume" => {
            let ok = require_client()?.resume().await?;
            Ok(serde_json::json!({ "resumed": ok }))
        }

        "speed-limit" => {
            let kbps = params
                .get("kbps")
                .and_then(Value::as_u64)
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `kbps`".to_string(),
                    param: "kbps".to_string(),
                })?;
            let ok = require_client()?.set_speed_limit(kbps).await?;
            Ok(serde_json::json!({ "speed_limit_set": ok }))
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `sabnzbd`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
