//! UniFi network actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::services::error::ToolError;

use super::helpers::{object_without, query_from, require_client, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "networks.list",
        description: "List networks for one site",
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
        name: "network.get",
        description: "Inspect one network",
        destructive: false,
        returns: "Network",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
    ActionSpec {
        name: "network.references",
        description: "List references to a network (VLANs, Wi-Fi, etc.)",
        destructive: false,
        returns: "References",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
    ActionSpec {
        name: "networks.create",
        description: "Create a network",
        destructive: true,
        returns: "Network",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "networks.update",
        description: "Update a network",
        destructive: true,
        returns: "Network",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
    ActionSpec {
        name: "networks.delete",
        description: "Delete a network",
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
                name: "network_id",
                ty: "string",
                required: true,
                description: "Network UUID",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "networks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let networks = if q.is_empty() {
                require_client()?.networks_list(site_id).await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/networks"), &q)
                    .await?
            };
            to_json(networks)
        }
        "network.get" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let network = require_client()?.network_get(site_id, network_id).await?;
            to_json(network)
        }
        "network.references" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let refs = require_client()?
                .network_references(site_id, network_id)
                .await?;
            to_json(refs)
        }
        "networks.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/networks"), &body)
                .await?;
            to_json(result)
        }
        "networks.update" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let body = object_without(&params, &["site_id", "network_id"])?;
            let result = require_client()?
                .put_value(&format!("/sites/{site_id}/networks/{network_id}"), &body)
                .await?;
            to_json(result)
        }
        "networks.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/networks/{network_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => unreachable!(),
    }
}
