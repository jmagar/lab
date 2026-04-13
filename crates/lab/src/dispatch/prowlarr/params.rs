use lab_apis::prowlarr::types::HistoryQuery;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, require_i64};

/// Extract a required integer `id` parameter and return it as `i64`.
pub fn require_id(params: &Value) -> Result<i64, ToolError> {
    require_i64(params, "id")
}

/// Extract optional history query parameters from params.
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
    if let Some(p) = page {
        if p < 1 {
            return Err(ToolError::InvalidParam {
                message: "parameter `page` must be >= 1 (pages are 1-based)".to_string(),
                param: "page".to_string(),
            });
        }
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
