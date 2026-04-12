//! `UniFi` traffic matching list actions.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use serde_json::Value;

use crate::dispatch::error::ToolError;

use super::params::{object_without, query_from, require_str, to_json};
use lab_apis::unifi::UnifiClient;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "traffic-matching-lists.list",
        description: "List traffic matching lists for one site",
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
        name: "traffic-matching-lists.get",
        description: "Inspect one traffic matching list",
        destructive: false,
        returns: "TrafficMatchingList",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
            },
        ],
    },
    ActionSpec {
        name: "traffic-matching-lists.create",
        description: "Create a traffic matching list",
        destructive: true,
        returns: "TrafficMatchingList",
        params: &[ParamSpec {
            name: "site_id",
            ty: "string",
            required: true,
            description: "Site UUID",
        }],
    },
    ActionSpec {
        name: "traffic-matching-lists.update",
        description: "Update a traffic matching list",
        destructive: true,
        returns: "TrafficMatchingList",
        params: &[
            ParamSpec {
                name: "site_id",
                ty: "string",
                required: true,
                description: "Site UUID",
            },
            ParamSpec {
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
            },
        ],
    },
    ActionSpec {
        name: "traffic-matching-lists.delete",
        description: "Delete a traffic matching list",
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
                name: "traffic_matching_list_id",
                ty: "string",
                required: true,
                description: "Traffic matching list UUID",
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
        "traffic-matching-lists.list" => {
            let site_id = require_str(&params, "site_id")?;
            let q = query_from(&params, &["offset", "limit", "filter"])?;
            let lists = if q.is_empty() {
                client
                    .get_value(&format!("/sites/{site_id}/traffic-matching-lists"))
                    .await?
            } else {
                client
                    .get_value_query(&format!("/sites/{site_id}/traffic-matching-lists"), &q)
                    .await?
            };
            to_json(lists)
        }
        "traffic-matching-lists.get" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            let list = client
                .get_value(&format!(
                    "/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"
                ))
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.create" => {
            let site_id = require_str(&params, "site_id")?;
            let body = object_without(&params, &["site_id"])?;
            let list = client
                .post_value(&format!("/sites/{site_id}/traffic-matching-lists"), &body)
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.update" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            let body = object_without(&params, &["site_id", "traffic_matching_list_id"])?;
            let list = client
                .put_value(
                    &format!("/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"),
                    &body,
                )
                .await?;
            to_json(list)
        }
        "traffic-matching-lists.delete" => {
            let site_id = require_str(&params, "site_id")?;
            let traffic_matching_list_id = require_str(&params, "traffic_matching_list_id")?;
            client
                .delete_value(&format!(
                    "/sites/{site_id}/traffic-matching-lists/{traffic_matching_list_id}"
                ))
                .await?;
            to_json(serde_json::json!({"deleted": true}))
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `unifi`"),
            valid: vec![],
            hint: None,
        }),
    }
}
