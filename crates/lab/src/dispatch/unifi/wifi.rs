//! UniFi Wi-Fi broadcast actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::client::require_client;
use super::params::{object_without, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "wifi.broadcasts.list",
        description: "List Wi-Fi broadcasts (SSIDs) for one site",
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
        name: "wifi.broadcasts.get",
        description: "Inspect one Wi-Fi broadcast",
        destructive: false,
        returns: "WifiBroadcast",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "wifi_broadcast_id",
                ty: "string",
                required: true,
                description: "Wi-Fi broadcast UUID",
            },
        ],
    },
    ActionSpec {
        name: "wifi.broadcasts.create",
        description: "Create a Wi-Fi broadcast",
        destructive: true,
        returns: "WifiBroadcast",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "wifi.broadcasts.update",
        description: "Update a Wi-Fi broadcast",
        destructive: true,
        returns: "WifiBroadcast",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "wifi_broadcast_id",
                ty: "string",
                required: true,
                description: "Wi-Fi broadcast UUID",
            },
        ],
    },
    ActionSpec {
        name: "wifi.broadcasts.delete",
        description: "Delete a Wi-Fi broadcast",
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
                name: "wifi_broadcast_id",
                ty: "string",
                required: true,
                description: "Wi-Fi broadcast UUID",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "wifi.broadcasts.list" => {
            let site_id = require_str(&params, "site_id")?;
            let broadcasts = require_client()?
                .get_value(&format!("/sites/{site_id}/wifi/broadcasts"))
                .await?;
            to_json(broadcasts)
        }
        "wifi.broadcasts.get" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            let broadcast = require_client()?
                .get_value(&format!(
                    "/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"
                ))
                .await?;
            to_json(broadcast)
        }
        "wifi.broadcasts.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/wifi/broadcasts"), &body)
                .await?;
            to_json(result)
        }
        "wifi.broadcasts.update" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            let body = object_without(&params, &["site_id", "wifi_broadcast_id"])?;
            let result = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        "wifi.broadcasts.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let wifi_broadcast_id = require_str(&params, "wifi_broadcast_id")?;
            require_client()?
                .delete_value(&format!(
                    "/sites/{site_id}/wifi/broadcasts/{wifi_broadcast_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => unreachable!(),
    }
}
