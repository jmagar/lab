//! Queue resource dispatch: list, remove.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::types::QueueItemId;
use serde_json::Value;

use super::params::{require_i64, to_json};
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "queue.list",
        description: "List all items currently in the download queue",
        destructive: false,
        returns: "QueueItem[]",
        params: &[],
    },
    ActionSpec {
        name: "queue.remove",
        description: "Remove an item from the download queue",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Queue item ID",
            },
            ParamSpec {
                name: "remove_from_client",
                ty: "bool",
                required: false,
                description: "Remove from download client too (default true)",
            },
            ParamSpec {
                name: "blocklist",
                ty: "bool",
                required: false,
                description: "Add release to blocklist (default false)",
            },
        ],
    },
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "queue.list" => {
            let items = client.queue_list().await?;
            to_json(items)
        }
        "queue.remove" => {
            let id = QueueItemId(require_i64(&params, "id")?);
            let remove_from_client = params
                .get("remove_from_client")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let blocklist = params
                .get("blocklist")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            client
                .queue_remove(id, remove_from_client, blocklist)
                .await?;
            Ok(serde_json::json!({ "removed": true }))
        }
        _ => unreachable!(),
    }
}
