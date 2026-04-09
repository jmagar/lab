//! UniFi DNS policy actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::services::error::ToolError;

use super::helpers::{object_without, query_from, require_client, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "dns.policies.list",
        description: "List DNS policies for one site",
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
        name: "dns.policies.get",
        description: "Inspect one DNS policy",
        destructive: false,
        returns: "DnsPolicy",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "dns.policies.create",
        description: "Create a DNS policy",
        destructive: true,
        returns: "DnsPolicy",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "dns.policies.update",
        description: "Update a DNS policy",
        destructive: true,
        returns: "DnsPolicy",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
    ActionSpec {
        name: "dns.policies.delete",
        description: "Delete a DNS policy",
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
                name: "dns_policy_id",
                ty: "string",
                required: true,
                description: "DNS policy UUID",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "dns.policies.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let policies = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/dns/policies"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/dns/policies"), &q)
                    .await?
            };
            to_json(policies)
        }
        "dns.policies.get" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            let policy = require_client()?
                .get_value(&format!("/sites/{site_id}/dns/policies/{dns_policy_id}"))
                .await?;
            to_json(policy)
        }
        "dns.policies.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let policy = require_client()?
                .post_value(&format!("/sites/{site_id}/dns/policies"), &body)
                .await?;
            to_json(policy)
        }
        "dns.policies.update" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            let body = object_without(&params, &["site_id", "dns_policy_id"])?;
            let policy = require_client()?
                .put_value(
                    &format!("/sites/{site_id}/dns/policies/{dns_policy_id}"),
                    &body,
                )
                .await?;
            to_json(policy)
        }
        "dns.policies.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let dns_policy_id = require_str(&params, "dns_policy_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/dns/policies/{dns_policy_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => unreachable!(),
    }
}
