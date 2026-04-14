//! System resource dispatch: status, health, disk-space, logs, updates,
//! restart, backup, tasks, and task execution.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use super::params::{require_str, to_json};
use crate::dispatch::error::ToolError;

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
    ActionSpec {
        name: "system.restart",
        description: "Restart the Radarr application",
        destructive: true,
        returns: "void",
        params: &[],
    },
    ActionSpec {
        name: "system.backup",
        description: "List available Radarr backup files",
        destructive: false,
        returns: "BackupFile[]",
        params: &[],
    },
    ActionSpec {
        name: "system.task",
        description: "List all scheduled background tasks",
        destructive: false,
        returns: "Task[]",
        params: &[],
    },
    ActionSpec {
        name: "system.task-execute",
        description: "Execute a named scheduled task immediately",
        destructive: true,
        returns: "Command",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Task name to execute (e.g. RssSync, RefreshMonitoredDownloads)",
        }],
    },
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "system.status" => {
            let status = client.system_status().await?;
            to_json(status)
        }
        "system.health" => {
            let checks = client.health_checks().await?;
            to_json(checks)
        }
        "system.disk-space" => {
            let disks = client.disk_space().await?;
            to_json(disks)
        }
        "system.logs" => {
            let logs = client.log_files().await?;
            to_json(logs)
        }
        "system.updates" => {
            let updates = client.updates().await?;
            to_json(updates)
        }
        "system.restart" => {
            client.system_restart().await?;
            Ok(serde_json::json!({ "restarting": true }))
        }
        "system.backup" => {
            let backups = client.system_backup().await?;
            Ok(backups)
        }
        "system.task" => {
            let tasks = client.system_tasks().await?;
            Ok(tasks)
        }
        "system.task-execute" => {
            let name = require_str(&params, "name")?;
            let cmd = client.system_task_execute(name).await?;
            to_json(cmd)
        }
        _ => unreachable!(),
    }
}
