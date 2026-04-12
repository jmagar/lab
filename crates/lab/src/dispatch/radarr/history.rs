//! History and blocklist dispatch.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use super::params::to_json;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::optional_u32;

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
            let page = optional_u32(&params, "page")?.unwrap_or(1);
            let page_size = optional_u32(&params, "page_size")?.unwrap_or(10);
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
