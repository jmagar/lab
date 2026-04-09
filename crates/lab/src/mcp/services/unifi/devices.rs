//! UniFi device and pending-device actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::services::error::ToolError;

use super::helpers::{object_without, require_client, require_i64, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "devices.list",
        description: "List adopted devices for one site",
        destructive: false,
        returns: "Page",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "devices.get",
        description: "Inspect one adopted device",
        destructive: false,
        returns: "Device",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
        ],
    },
    ActionSpec {
        name: "devices.stats",
        description: "Get latest statistics for one adopted device",
        destructive: false,
        returns: "DeviceStats",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
        ],
    },
    ActionSpec {
        name: "pending-devices.list",
        description: "List devices pending adoption",
        destructive: false,
        returns: "Page",
        params: &[],
    },
    ActionSpec {
        name: "devices.create",
        description: "Adopt a device into a site",
        destructive: true,
        returns: "Device",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "macAddress",
                ty: "string",
                required: true,
                description: "MAC address of the device",
            },
            ParamSpec {
                name: "ignoreDeviceLimit",
                ty: "boolean",
                required: false,
                description: "Override device limit",
            },
        ],
    },
    ActionSpec {
        name: "devices.port-action",
        description: "Perform an action on one port of a device",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
            ParamSpec {
                name: "port_idx",
                ty: "integer",
                required: true,
                description: "Port index",
            },
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Action to perform (e.g. power_cycle)",
            },
        ],
    },
    ActionSpec {
        name: "devices.action",
        description: "Perform an action on a device (restart, etc.)",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Action to perform",
            },
        ],
    },
    ActionSpec {
        name: "devices.delete",
        description: "Remove (forget) an adopted device",
        destructive: true,
        returns: "Confirmation",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "device_id",
                ty: "string",
                required: true,
                description: "Device UUID",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "devices.list" => {
            let site_id = require_str(&params, "site_id")?;
            let devices = require_client()?.devices_list(site_id).await?;
            to_json(devices)
        }
        "devices.get" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let device = require_client()?.device_get(site_id, device_id).await?;
            to_json(device)
        }
        "devices.stats" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let stats = require_client()?
                .device_stats_latest(site_id, device_id)
                .await?;
            to_json(stats)
        }
        "pending-devices.list" => {
            let pending = require_client()?.pending_devices_list().await?;
            to_json(pending)
        }
        "devices.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/devices"), &body)
                .await?;
            to_json(result)
        }
        "devices.port-action" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let port_idx = require_i64(&params, "port_idx")?;
            let body = object_without(&params, &["site_id", "device_id", "port_idx"])?;
            let result = require_client()?
                .post_value(
                    &format!(
                        "/sites/{site_id}/devices/{device_id}/interfaces/ports/{port_idx}/actions"
                    ),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "devices.action" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            let body = object_without(&params, &["site_id", "device_id"])?;
            let result = require_client()?
                .post_value(
                    &format!("/sites/{site_id}/devices/{device_id}/actions"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "devices.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let device_id = require_str(&params, "device_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/devices/{device_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => unreachable!(),
    }
}
