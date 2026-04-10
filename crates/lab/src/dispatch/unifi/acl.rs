//! UniFi ACL rule actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::client::require_client;
use super::params::{object_without, query_from, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "acl.rules.list",
        description: "List ACL rules for one site",
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
        name: "acl.rules.get",
        description: "Inspect one ACL rule",
        destructive: false,
        returns: "AclRule",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.create",
        description: "Create an ACL rule",
        destructive: true,
        returns: "AclRule",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "acl.rules.update",
        description: "Update an ACL rule",
        destructive: true,
        returns: "AclRule",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.delete",
        description: "Delete an ACL rule",
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
                name: "acl_rule_id",
                ty: "string",
                required: true,
                description: "ACL rule UUID",
            },
        ],
    },
    ActionSpec {
        name: "acl.rules.ordering.get",
        description: "Get ACL rule ordering",
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
        name: "acl.rules.ordering.set",
        description: "Set ACL rule ordering",
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

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "acl.rules.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let rules = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/acl-rules"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/acl-rules"), &q)
                    .await?
            };
            to_json(rules)
        }
        "acl.rules.get" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            let rule = require_client()?
                .get_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"))
                .await?;
            to_json(rule)
        }
        "acl.rules.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let rule = require_client()?
                .post_value(&format!("/sites/{site_id}/acl-rules"), &body)
                .await?;
            to_json(rule)
        }
        "acl.rules.update" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            let body = object_without(&params, &["site_id", "acl_rule_id"])?;
            let rule = require_client()?
                .put_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"), &body)
                .await?;
            to_json(rule)
        }
        "acl.rules.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let acl_rule_id = require_str(&params, "acl_rule_id")?;
            require_client()?
                .delete_value(&format!("/sites/{site_id}/acl-rules/{acl_rule_id}"))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "acl.rules.ordering.get" => {
            let site_id = require_str(&params, "site_id")?;
            let ordering = require_client()?
                .get_value(&format!("/sites/{site_id}/acl-rules/ordering"))
                .await?;
            to_json(ordering)
        }
        "acl.rules.ordering.set" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let ordering = require_client()?
                .put_value(&format!("/sites/{site_id}/acl-rules/ordering"), &body)
                .await?;
            to_json(ordering)
        }
        _ => unreachable!(),
    }
}
