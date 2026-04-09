//! System resource dispatch: status, health, disk-space, logs, updates.

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use super::helpers::{require_client, to_json};
use crate::services::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "system.status",
        description: "Return Radarr system status and version",
        destructive: false,
        returns: "SystemStatus",
        params: &[],
    },
    ActionSpec {
        name: "system.health",
        description: "Return Radarr health check results",
        destructive: false,
        returns: "HealthCheck[]",
        params: &[],
    },
    ActionSpec {
        name: "system.disk-space",
        description: "Return disk space information for all drives",
        destructive: false,
        returns: "DiskSpace[]",
        params: &[],
    },
    ActionSpec {
        name: "system.logs",
        description: "Return list of available log files",
        destructive: false,
        returns: "LogFile[]",
        params: &[],
    },
    ActionSpec {
        name: "system.updates",
        description: "Return available Radarr updates",
        destructive: false,
        returns: "UpdateInfo[]",
        params: &[],
    },
];

pub async fn dispatch(action: &str, _params: Value) -> Result<Value, ToolError> {
    match action {
        "system.status" => {
            let status = require_client()?.system_status().await?;
            to_json(status)
        }
        "system.health" => {
            let checks = require_client()?.health_checks().await?;
            to_json(checks)
        }
        "system.disk-space" => {
            let disks = require_client()?.disk_space().await?;
            to_json(disks)
        }
        "system.logs" => {
            let logs = require_client()?.log_files().await?;
            to_json(logs)
        }
        "system.updates" => {
            let updates = require_client()?.updates().await?;
            to_json(updates)
        }
        _ => unreachable!(),
    }
}
