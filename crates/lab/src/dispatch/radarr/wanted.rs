//! Wanted resource dispatch: missing and below-cutoff movies.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::optional_u32;

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "wanted.missing",
        description: "List monitored movies that are missing from the library",
        destructive: false,
        returns: "WantedPage",
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
        name: "wanted.cutoff",
        description: "List monitored movies that do not meet the quality cutoff",
        destructive: false,
        returns: "WantedPage",
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
];

pub async fn dispatch_with_client(
    client: &RadarrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "wanted.missing" => {
            let page = optional_u32(&params, "page")?.unwrap_or(1);
            let page_size = optional_u32(&params, "page_size")?.unwrap_or(10);
            let result = client.wanted_missing(page, page_size).await?;
            Ok(result)
        }
        "wanted.cutoff" => {
            let page = optional_u32(&params, "page")?.unwrap_or(1);
            let page_size = optional_u32(&params, "page_size")?.unwrap_or(10);
            let result = client.wanted_cutoff(page, page_size).await?;
            Ok(result)
        }
        _ => unreachable!(),
    }
}
