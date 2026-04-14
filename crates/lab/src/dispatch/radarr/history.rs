//! History and blocklist dispatch.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::types::history::{BlocklistId, HistoryId};
use lab_apis::radarr::types::movie::MovieId;
use serde_json::Value;

use super::params::to_json;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_u32, require_i64};

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
        name: "history.movie",
        description: "List history records for a specific movie",
        destructive: false,
        returns: "HistoryRecord[]",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Radarr movie ID",
        }],
    },
    ActionSpec {
        name: "history.failed-retry",
        description: "Mark a history record as failed and trigger a retry search",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "History record ID to mark failed and retry",
        }],
    },
    ActionSpec {
        name: "blocklist.list",
        description: "List blocked releases",
        destructive: false,
        returns: "BlocklistItem[]",
        params: &[],
    },
    ActionSpec {
        name: "blocklist.delete",
        description: "Delete a specific blocklist entry",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Blocklist entry ID to delete",
        }],
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
        "history.movie" => {
            let id = MovieId(require_i64(&params, "id")?);
            let records = client.history_movie(id).await?;
            Ok(records)
        }
        "history.failed-retry" => {
            let id = HistoryId(require_i64(&params, "id")?);
            client.history_failed_retry(id).await?;
            Ok(serde_json::json!({ "retried": true }))
        }
        "blocklist.list" => {
            let items = client.blocklist_list().await?;
            to_json(items)
        }
        "blocklist.delete" => {
            let id = BlocklistId(require_i64(&params, "id")?);
            client.blocklist_delete(id).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }
        _ => unreachable!(),
    }
}
