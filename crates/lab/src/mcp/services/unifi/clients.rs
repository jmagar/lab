//! UniFi client actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::services::error::ToolError;

use super::helpers::{object_without, require_client, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "clients.list",
        description: "List active clients for one site",
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
        name: "clients.get",
        description: "Inspect one client",
        destructive: false,
        returns: "Client",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "client_id",
                ty: "string",
                required: true,
                description: "Client UUID",
            },
        ],
    },
    ActionSpec {
        name: "clients.action",
        description: "Perform an action on a client (block, reconnect, etc.)",
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
                name: "client_id",
                ty: "string",
                required: true,
                description: "Client UUID",
            },
            ParamSpec {
                name: "action",
                ty: "string",
                required: true,
                description: "Action to perform",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "clients.list" => {
            let site_id = require_str(&params, "site_id")?;
            let clients = require_client()?.clients_list(site_id).await?;
            to_json(clients)
        }
        "clients.get" => {
            let site_id = require_str(&params, "site_id")?;
            let client_id = require_str(&params, "client_id")?;
            let client = require_client()?.client_get(site_id, client_id).await?;
            to_json(client)
        }
        "clients.action" => {
            let site_id = require_str(&params, "site_id")?;
            let client_id = require_str(&params, "client_id")?;
            let body = object_without(&params, &["site_id", "client_id"])?;
            let result = require_client()?
                .post_value(
                    &format!("/sites/{site_id}/clients/{client_id}/actions"),
                    &body,
                )
                .await?;
            to_json(result)
        }
        _ => unreachable!(),
    }
}
