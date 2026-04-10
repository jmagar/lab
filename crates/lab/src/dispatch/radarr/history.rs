//! History and blocklist dispatch.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use super::client::require_client;
use super::params::to_json;
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "history.list",
        description: "List download history",
        destructive: false,
        returns: "HistoryPage",
        params: &[
            ParamSpec {
                name: "page",
                ty: "u32",
                required: false,
                description: "Page number (default 1)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Items per page (default 10)",
            },
        ],
    },
    ActionSpec {
        name: "blocklist.list",
        description: "List blocked releases",
        destructive: false,
        returns: "BlocklistItem[]",
        params: &[],
    },
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "history.list" => {
            let page = u32::try_from(params.get("page").and_then(Value::as_u64).unwrap_or(1))
                .unwrap_or(u32::MAX);
            let page_size = u32::try_from(
                params
                    .get("page_size")
                    .and_then(Value::as_u64)
                    .unwrap_or(10),
            )
            .unwrap_or(u32::MAX);
            let page_data = client.history_list(page, page_size).await?;
            to_json(page_data)
        }
        "blocklist.list" => {
            let items = client.blocklist_list().await?;
            to_json(items)
        }
        _ => unreachable!(),
    }
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    dispatch_with_client(&require_client()?, action, params).await
}
