//! `UniFi` network actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::params::{object_without, query_from, require_str, to_json};
use lab_apis::unifi::UnifiClient;

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
        name: "networks.get",
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
        name: "networks.references",
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

pub async fn dispatch(
    client: &UnifiClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "networks.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let networks = if q.is_empty() {
                client.networks_list(site_id).await?
            } else {
                client
                    .get_value_query(&format!("/sites/{site_id}/networks"), &q)
                    .await?
            };
            to_json(networks)
        }
        "networks.get" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let network = client.network_get(site_id, network_id).await?;
            to_json(network)
        }
        "networks.references" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let refs = client.network_references(site_id, network_id).await?;
            to_json(refs)
        }
        "networks.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = client
                .post_value(&format!("/sites/{site_id}/networks"), &body)
                .await?;
            to_json(result)
        }
        "networks.update" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            let body = object_without(&params, &["site_id", "network_id"])?;
            let result = client
                .put_value(&format!("/sites/{site_id}/networks/{network_id}"), &body)
                .await?;
            to_json(result)
        }
        "networks.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let network_id = require_str(&params, "network_id")?;
            client
                .delete_value(&format!("/sites/{site_id}/networks/{network_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unifi`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
