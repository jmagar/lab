//! Param coercion helpers for the `Sonarr` dispatch layer.
//!
//! All `Value` → typed coercions live here. Dispatch arms call these helpers
//! rather than extracting params inline.

use serde_json::Value;

use lab_apis::sonarr::types::{AddSeriesRequest, CalendarQuery, HistoryQuery, QueueListQuery};

use crate::dispatch::error::ToolError;
pub use crate::dispatch::helpers::{optional_u32, require_i64, require_str};

/// Extract an optional i64 from params.
pub fn optional_i64(params: &Value, key: &str) -> Result<Option<i64>, ToolError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => {
            let n = v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be an integer"),
                param: key.to_string(),
            })?;
            Ok(Some(n))
        }
    }
}

/// Build an `AddSeriesRequest` from dispatch params.
pub fn add_series_from_params(params: &Value) -> Result<AddSeriesRequest, ToolError> {
    let tvdb_id = require_i64(params, "tvdb_id")?;
    let title = require_str(params, "title")?.to_owned();
    let quality_profile_id = require_i64(params, "quality_profile_id")?;
    let language_profile_id = require_i64(params, "language_profile_id")?;
    let root_folder_path = require_str(params, "root_folder_path")?.to_owned();
    let monitored = params
        .get("monitored")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let series_type = params
        .get("series_type")
        .and_then(Value::as_str)
        .unwrap_or("standard")
        .to_owned();

    Ok(AddSeriesRequest {
        title,
        tvdb_id,
        quality_profile_id,
        language_profile_id,
        root_folder_path,
        monitored,
        season_folder: true,
        series_type,
        add_options: serde_json::json!({
            "searchForMissingEpisodes": monitored,
            "monitor": "all"
        }),
    })
}

/// Build a `QueueListQuery` from dispatch params.
pub fn queue_query_from_params(params: &Value) -> Result<QueueListQuery, ToolError> {
    Ok(QueueListQuery {
        page: optional_u32(params, "page")?,
        page_size: optional_u32(params, "page_size")?,
        series_id: optional_i64(params, "series_id")?,
    })
}

/// Build a `HistoryQuery` from dispatch params.
pub fn history_query_from_params(params: &Value) -> Result<HistoryQuery, ToolError> {
    Ok(HistoryQuery {
        page: optional_u32(params, "page")?,
        page_size: optional_u32(params, "page_size")?,
        series_id: optional_i64(params, "series_id")?,
        episode_id: optional_i64(params, "episode_id")?,
    })
}

/// Extract a list of episode IDs from params.
///
/// Expects `episode_ids` to be a JSON array of integers.
pub fn episode_ids_from_params(params: &Value) -> Result<Vec<i64>, ToolError> {
    let arr = params
        .get("episode_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::MissingParam {
            param: "episode_ids".to_string(),
            message: "parameter `episode_ids` must be an array of integers".to_string(),
        })?;
    arr.iter()
        .map(|v| {
            v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: "each element of `episode_ids` must be an integer".to_string(),
                param: "episode_ids".to_string(),
            })
        })
        .collect()
}

/// Build a `CalendarQuery` from dispatch params.
#[allow(clippy::unnecessary_wraps)]
pub fn calendar_query_from_params(params: &Value) -> Result<CalendarQuery, ToolError> {
    Ok(CalendarQuery {
        start: params
            .get("start")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        end: params
            .get("end")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        unmonitored: params.get("unmonitored").and_then(Value::as_bool),
    })
}
