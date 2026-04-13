use lab_apis::tautulli::types::HistoryQuery;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, require_i64};

/// Extract a required integer `user_id` param.
pub fn require_user_id(params: &Value) -> Result<i64, ToolError> {
    require_i64(params, "user_id")
}

/// Extract a required integer `section_id` param.
pub fn require_section_id(params: &Value) -> Result<i64, ToolError> {
    require_i64(params, "section_id")
}

/// Extract optional history query parameters.
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

    let page_size = params.get("page_size").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `page_size` must be a non-negative integer".to_string(),
                param: "page_size".to_string(),
            })
    })?;

    let order_dir = optional_str(params, "order_dir")?.map(str::to_owned);
    let media_type = optional_str(params, "media_type")?.map(str::to_owned);

    let user_id = params.get("user_id").map_or(Ok(None), |v| {
        v.as_i64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `user_id` must be an integer".to_string(),
                param: "user_id".to_string(),
            })
    })?;

    let section_id = params.get("section_id").map_or(Ok(None), |v| {
        v.as_i64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `section_id` must be an integer".to_string(),
                param: "section_id".to_string(),
            })
    })?;

    let rating_key = params.get("rating_key").map_or(Ok(None), |v| {
        v.as_i64()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `rating_key` must be an integer".to_string(),
                param: "rating_key".to_string(),
            })
    })?;

    Ok(HistoryQuery {
        page,
        page_size,
        order_dir,
        media_type,
        user_id,
        section_id,
        rating_key,
    })
}

/// Extract optional `time_range` and `stats_count` for home stats.
pub fn home_stats_params(params: &Value) -> Result<(Option<u32>, Option<u32>), ToolError> {
    let time_range = params.get("time_range").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `time_range` must be a non-negative integer".to_string(),
                param: "time_range".to_string(),
            })
    })?;

    let stats_count = params.get("stats_count").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `stats_count` must be a non-negative integer".to_string(),
                param: "stats_count".to_string(),
            })
    })?;

    Ok((time_range, stats_count))
}

/// Extract optional `time_range` and `y_axis` for plays-by-date.
pub fn plays_by_date_params(
    params: &Value,
) -> Result<(Option<u32>, Option<String>), ToolError> {
    let time_range = params.get("time_range").map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `time_range` must be a non-negative integer".to_string(),
                param: "time_range".to_string(),
            })
    })?;

    let y_axis = optional_str(params, "y_axis")?.map(str::to_owned);

    Ok((time_range, y_axis))
}
