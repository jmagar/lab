use serde_json::Value;

use lab_apis::linkding::types::{
    BookmarkListParams, BookmarkUpdateRequest, BookmarkWriteRequest, TagCreateRequest,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{body_from_params, optional_str, optional_u32, require_i64};

/// Extract a required `id` param as `u64`, rejecting negative values.
pub fn require_id_u64(params: &Value) -> Result<u64, ToolError> {
    let n = require_i64(params, "id")?;
    u64::try_from(n).map_err(|_| ToolError::InvalidParam {
        message: "id must be non-negative".to_string(),
        param: "id".to_string(),
    })
}

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
///
/// Returns `ToolError::InvalidParam` when no updatable fields are provided, since
/// sending an empty PATCH body would be a no-op.
pub fn bookmark_update_from_params(params: &Value) -> Result<BookmarkUpdateRequest, ToolError> {
    let req: BookmarkUpdateRequest =
        serde_json::from_value(body_from_params(params, &["payload", "body", "id"])).map_err(
            |e| ToolError::InvalidParam {
                message: e.to_string(),
                param: "payload".to_string(),
            },
        )?;

    if req.url.is_none()
        && req.title.is_none()
        && req.description.is_none()
        && req.notes.is_none()
        && req.unread.is_none()
        && req.shared.is_none()
        && req.tag_names.is_none()
    {
        return Err(ToolError::InvalidParam {
            message: "at least one field must be provided for bookmark update".into(),
            param: "params".into(),
        });
    }

    Ok(req)
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
