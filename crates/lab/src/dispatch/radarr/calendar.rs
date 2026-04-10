//! Calendar resource dispatch.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use super::params::to_json;
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[ActionSpec] = &[ActionSpec {
    name: "calendar.list",
    description: "List upcoming movie releases",
    destructive: false,
    returns: "CalendarEntry[]",
    params: &[
        ParamSpec {
            name: "start",
            ty: "string",
            required: false,
            description: "Start date ISO 8601 (default today)",
        },
        ParamSpec {
            name: "end",
            ty: "string",
            required: false,
            description: "End date ISO 8601 (default 7 days from now)",
        },
    ],
}];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "calendar.list" => {
            let start = params.get("start").and_then(Value::as_str);
            let end = params.get("end").and_then(Value::as_str);
            let entries = client.calendar_list(start, end).await?;
            to_json(entries)
        }
        _ => unreachable!(),
    }
}

