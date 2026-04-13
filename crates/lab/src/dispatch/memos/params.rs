use serde_json::Value;

use lab_apis::memos::types::{CreateMemoRequest, ListMemosParams, UpdateMemoRequest};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, optional_u32, require_str};

/// Extract `ListMemosParams` from dispatch params value.
pub fn list_params_from(params: &Value) -> Result<ListMemosParams, ToolError> {
    let creator = optional_str(params, "creator")?.map(ToOwned::to_owned);
    let tag = optional_str(params, "tag")?.map(ToOwned::to_owned);
    let page_size = optional_u32(params, "page_size")?;
    let page_token = optional_str(params, "page_token")?.map(ToOwned::to_owned);
    Ok(ListMemosParams {
        creator,
        tag,
        page_size,
        page_token,
    })
}

/// Extract a required `name` param (memo resource name).
pub fn require_name<'a>(params: &'a Value) -> Result<&'a str, ToolError> {
    require_str(params, "name")
}

/// Build a `CreateMemoRequest` from dispatch params.
pub fn create_request_from(params: &Value) -> Result<CreateMemoRequest, ToolError> {
    let content = require_str(params, "content")?.to_owned();
    let visibility = optional_str(params, "visibility")?
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "PRIVATE".to_string());
    Ok(CreateMemoRequest { content, visibility })
}

/// Build an `UpdateMemoRequest` from dispatch params.
///
/// Returns `ToolError::InvalidParam` when no updatable fields are provided.
pub fn update_request_from(params: &Value) -> Result<UpdateMemoRequest, ToolError> {
    let content = optional_str(params, "content")?.map(ToOwned::to_owned);
    let visibility = optional_str(params, "visibility")?.map(ToOwned::to_owned);
    if content.is_none() && visibility.is_none() {
        return Err(ToolError::InvalidParam {
            message: "at least one field must be provided for memo update (content or visibility)"
                .into(),
            param: "params".into(),
        });
    }
    Ok(UpdateMemoRequest { content, visibility })
}
