use serde_json::Value;

use lab_apis::linkding::types::{BookmarkListParams, BookmarkUpdateRequest, BookmarkWriteRequest, TagCreateRequest};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{body_from_params, optional_u32, optional_str};

/// Extract optional bookmark list query params from a dispatch params value.
pub fn bookmark_list_params_from(params: &Value) -> Result<BookmarkListParams, ToolError> {
    let q = optional_str(params, "q")?.map(ToOwned::to_owned);
    let limit = optional_u32(params, "limit")?;
    let offset = optional_u32(params, "offset")?;
    Ok(BookmarkListParams { q, limit, offset })
}

/// Build a `BookmarkWriteRequest` from dispatch params.
///
/// Supports a `payload` or `body` key as a full JSON override, or individual keys.
pub fn bookmark_write_from_params(params: &Value) -> Result<BookmarkWriteRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Build a `BookmarkUpdateRequest` from dispatch params.
///
/// Supports a `payload` or `body` key as a full JSON override, or individual keys.
pub fn bookmark_update_from_params(params: &Value) -> Result<BookmarkUpdateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body", "id"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Build a `TagCreateRequest` from dispatch params.
///
/// Supports a `payload` or `body` key as a full JSON override, or a `name` key.
pub fn tag_create_from_params(params: &Value) -> Result<TagCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}
