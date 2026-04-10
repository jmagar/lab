//! UniFi hotspot voucher actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::client::require_client;
use super::params::{object_without, query_from, require_str, to_json};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "hotspot.vouchers.list",
        description: "List hotspot vouchers for one site",
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
        name: "hotspot.vouchers.create",
        description: "Create a hotspot voucher",
        destructive: true,
        returns: "Voucher",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "hotspot.vouchers.delete",
        description: "Delete hotspot vouchers (with optional filter)",
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
                name: "filter",
                ty: "string",
                required: false,
                description: "Filter expression",
            },
        ],
    },
    ActionSpec {
        name: "hotspot.vouchers.get",
        description: "Inspect one hotspot voucher",
        destructive: false,
        returns: "Voucher",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "voucher_id",
                ty: "string",
                required: true,
                description: "Voucher UUID",
            },
        ],
    },
];

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "hotspot.vouchers.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let vouchers = if q.is_empty() {
                require_client()?
                    .get_value(&format!("/sites/{site_id}/hotspot/vouchers"))
                    .await?
            } else {
                require_client()?
                    .get_value_query(&format!("/sites/{site_id}/hotspot/vouchers"), &q)
                    .await?
            };
            to_json(vouchers)
        }
        "hotspot.vouchers.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let result = require_client()?
                .post_value(&format!("/sites/{site_id}/hotspot/vouchers"), &body)
                .await?;
            to_json(result)
        }
        "hotspot.vouchers.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["filter"])?;
            require_client()?
                .delete_value_query(&format!("/sites/{site_id}/hotspot/vouchers"), &q)
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        "hotspot.vouchers.get" => {
            let site_id = require_str(&params, "site_id")?;
            let voucher_id = require_str(&params, "voucher_id")?;
            let voucher = require_client()?
                .get_value(&format!("/sites/{site_id}/hotspot/vouchers/{voucher_id}"))
                .await?;
            to_json(voucher)
        }
        _ => unreachable!(),
    }
}
