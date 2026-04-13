//! `UniFi` firewall zone and policy actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::params::{object_without, query_from, require_str, to_json};
use lab_apis::unifi::UnifiClient;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "firewall.zones.list",
        description: "List firewall zones for one site",
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
        name: "firewall.zones.get",
        description: "Inspect one firewall zone",
        destructive: false,
        returns: "FirewallZone",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.zones.create",
        description: "Create a firewall zone",
        destructive: true,
        returns: "FirewallZone",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.zones.update",
        description: "Update a firewall zone",
        destructive: true,
        returns: "FirewallZone",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.zones.delete",
        description: "Delete a firewall zone",
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
                name: "firewall_zone_id",
                ty: "string",
                required: true,
                description: "Firewall zone UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.list",
        description: "List firewall policies for one site",
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
        name: "firewall.policies.get",
        description: "Inspect one firewall policy",
        destructive: false,
        returns: "FirewallPolicy",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.create",
        description: "Create a firewall policy",
        destructive: true,
        returns: "FirewallPolicy",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.policies.update",
        description: "Replace a firewall policy",
        destructive: true,
        returns: "FirewallPolicy",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.patch",
        description: "Partially update a firewall policy",
        destructive: true,
        returns: "FirewallPolicy",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "firewall_policy_id",
                ty: "string",
                required: true,
                description: "Firewall policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "firewall.policies.ordering.get",
        description: "Get firewall policy ordering",
        destructive: false,
        returns: "Ordering",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "firewall.policies.ordering.set",
        description: "Set firewall policy ordering",
        destructive: true,
        returns: "Ordering",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
];

#[allow(clippy::too_many_lines)]
pub async fn dispatch(
    client: &UnifiClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "firewall.zones.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let zones = if q.is_empty() {
                client
                    .get_value(&format!("/sites/{site_id}/firewall/zones"))
                    .await?
            } else {
                client
                    .get_value_query(&format!("/sites/{site_id}/firewall/zones"), &q)
                    .await?
            };
            to_json(zones)
        }
        "firewall.zones.get" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            let zone = client
                .get_value(&format!(
                    "/sites/{site_id}/firewall/zones/{firewall_zone_id}"
                ))
                .await?;
            to_json(zone)
        }
        "firewall.zones.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let zone = client
                .post_value(&format!("/sites/{site_id}/firewall/zones"), &body)
                .await?;
            to_json(zone)
        }
        "firewall.zones.update" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            let body = object_without(&params, &["site_id", "firewall_zone_id"])?;
            let zone = client
                .put_value(
                    &format!("/sites/{site_id}/firewall/zones/{firewall_zone_id}"),
                    &body,
                )
                .await?;
            to_json(zone)
        }
        "firewall.zones.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_zone_id = require_str(&params, "firewall_zone_id")?;
            client
                .delete_value(&format!(
                    "/sites/{site_id}/firewall/zones/{firewall_zone_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "firewall.policies.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let policies = if q.is_empty() {
                client
                    .get_value(&format!("/sites/{site_id}/firewall/policies"))
                    .await?
            } else {
                client
                    .get_value_query(&format!("/sites/{site_id}/firewall/policies"), &q)
                    .await?
            };
            to_json(policies)
        }
        "firewall.policies.get" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let policy = client
                .get_value(&format!(
                    "/sites/{site_id}/firewall/policies/{firewall_policy_id}"
                ))
                .await?;
            to_json(policy)
        }
        "firewall.policies.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let policy = client
                .post_value(&format!("/sites/{site_id}/firewall/policies"), &body)
                .await?;
            to_json(policy)
        }
        "firewall.policies.update" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let body = object_without(&params, &["site_id", "firewall_policy_id"])?;
            let policy = client
                .put_value(
                    &format!("/sites/{site_id}/firewall/policies/{firewall_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "firewall.policies.patch" => {
            let site_id = require_str(&params, "site_id")?;
            let firewall_policy_id = require_str(&params, "firewall_policy_id")?;
            let body = object_without(&params, &["site_id", "firewall_policy_id"])?;
            let policy = client
                .patch_value(
                    &format!("/sites/{site_id}/firewall/policies/{firewall_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "firewall.policies.ordering.get" => {
            let site_id = require_str(&params, "site_id")?;
            let ordering = client
                .get_value(&format!("/sites/{site_id}/firewall/policies/ordering"))
                .await?;
            to_json(ordering)
        }
        "firewall.policies.ordering.set" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let ordering = client
                .put_value(
                    &format!("/sites/{site_id}/firewall/policies/ordering"),
                    &body,
                )
                .await?;
            to_json(ordering)
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unifi`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
