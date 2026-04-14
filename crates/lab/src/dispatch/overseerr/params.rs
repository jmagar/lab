//! Parameter coercion helpers for `Overseerr` dispatch actions.
//!
//! All `Value` → typed conversions live here so `dispatch.rs` stays clean.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_u32, require_i64, require_str};
use lab_apis::overseerr::types::{CreateIssueBody, CreateRequestBody};

/// Extract a required `u64` integer parameter.
pub fn require_u64(params: &Value, key: &str) -> Result<u64, ToolError> {
    let n = require_i64(params, key)?;
    u64::try_from(n).map_err(|_| ToolError::InvalidParam {
        message: format!("parameter `{key}` must be a non-negative integer"),
        param: key.to_string(),
    })
}

/// Extract an optional `u64` integer parameter.
pub fn optional_u64(params: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    optional_u32(params, key)?.map_or(Ok(None), |n| Ok(Some(u64::from(n))))
}

/// Build a `CreateRequestBody` from dispatch params.
pub fn create_request_body(params: &Value) -> Result<CreateRequestBody, ToolError> {
    let media_type = require_str(params, "media_type")?.to_owned();
    let media_id = require_u64(params, "media_id")?;

    // Parse optional comma-separated seasons string into Vec<u32>.
    let seasons: Option<Vec<u32>> = params
        .get("seasons")
        .and_then(Value::as_str)
        .map(|s| {
            s.split(',')
                .filter(|p| !p.trim().is_empty())
                .map(|p| {
                    p.trim()
                        .parse::<u32>()
                        .map_err(|_| ToolError::InvalidParam {
                            message: "seasons must be comma-separated integers (e.g. '1,2,3')"
                                .into(),
                            param: "seasons".to_string(),
                        })
                })
                .collect::<Result<Vec<u32>, ToolError>>()
        })
        .transpose()?;

    let is4k = params.get("is4k").and_then(Value::as_bool);

    Ok(CreateRequestBody {
        media_type,
        media_id,
        seasons,
        is4k,
        server_id: None,
        profile_id: None,
        root_folder: None,
        language_profile_id: None,
        tags: None,
    })
}

/// Build a `CreateIssueBody` from dispatch params.
pub fn create_issue_body(params: &Value) -> Result<CreateIssueBody, ToolError> {
    let issue_type_raw = require_i64(params, "issue_type")?;
    let issue_type = u32::try_from(issue_type_raw).map_err(|_| ToolError::InvalidParam {
        message: "issue_type must be a positive integer (1-4)".into(),
        param: "issue_type".to_string(),
    })?;
    let message = require_str(params, "message")?.to_owned();
    let media_id = require_u64(params, "media_id")?;
    let problem_season = optional_u32(params, "problem_season")?;
    let problem_episode = optional_u32(params, "problem_episode")?;

    Ok(CreateIssueBody {
        issue_type,
        message,
        media_id,
        problem_season,
        problem_episode,
    })
}
