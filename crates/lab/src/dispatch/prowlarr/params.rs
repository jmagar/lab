use lab_apis::prowlarr::types::HistoryQuery;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, require_i64, require_str};

/// Extract a required integer `id` parameter and return it as `i64`.
pub fn require_id(params: &Value) -> Result<i64, ToolError> {
    require_i64(params, "id")
}

/// Extract a required string `guid` parameter.
pub fn require_guid(params: &Value) -> Result<String, ToolError> {
    require_str(params, "guid").map(str::to_owned)
}

/// Extract a required string `query` parameter.
pub fn require_query_str(params: &Value) -> Result<String, ToolError> {
    require_str(params, "query").map(str::to_owned)
}

/// Extract an optional string `search_type` parameter.
pub fn optional_search_type(params: &Value) -> Result<Option<String>, ToolError> {
    optional_str(params, "search_type").map(|opt| opt.map(str::to_owned))
}

/// Extract a required JSON object `body` parameter.
pub fn require_body(params: &Value) -> Result<Value, ToolError> {
    params
        .get("body")
        .cloned()
        .ok_or_else(|| ToolError::MissingParam {
            message: "parameter `body` is required".to_string(),
            param: "body".to_string(),
        })
}

/// Extract optional array-of-integer query params (indexer_ids, categories).
pub fn optional_i64_array(params: &Value, key: &str) -> Result<Vec<i64>, ToolError> {
    match params.get(key) {
        None => Ok(vec![]),
        Some(Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                let n = v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                    message: format!("each element of `{key}` must be an integer"),
                    param: key.to_string(),
                })?;
                out.push(n);
            }
            Ok(out)
        }
        Some(_) => Err(ToolError::InvalidParam {
            message: format!("parameter `{key}` must be an array of integers"),
            param: key.to_string(),
        }),
    }
}

/// Extract optional history query parameters from params.
#[allow(clippy::collapsible_if)]
pub fn history_query_from_params(params: &Value) -> Result<HistoryQuery, ToolError> {
    let page = params.get("page").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `page` must be a positive integer (1-based)".to_string(),
                param: "page".to_string(),
            })
    })?;
    if let Some(p) = page
        && p < 1
    {
        return Err(ToolError::InvalidParam {
            message: "parameter `page` must be >= 1 (pages are 1-based)".to_string(),
            param: "page".to_string(),
        });
    }

    let page_size = params.get("page_size").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `page_size` must be a non-negative integer".to_string(),
                param: "page_size".to_string(),
            })
    })?;

    let sort_key = optional_str(params, "sort_key")?.map(str::to_owned);
    let sort_dir = optional_str(params, "sort_dir")?.map(str::to_owned);

    let indexer_id = params.get("indexer_id").map_or(Ok(None), |v| {
        v.as_i64().map(Some).ok_or_else(|| ToolError::InvalidParam {
            message: "parameter `indexer_id` must be an integer".to_string(),
            param: "indexer_id".to_string(),
        })
    })?;

    Ok(HistoryQuery {
        page,
        page_size,
        sort_key,
        sort_dir,
        indexer_id,
    })
}
