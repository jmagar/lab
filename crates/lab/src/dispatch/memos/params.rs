use serde_json::Value;

use base64::Engine as _;
use lab_apis::memos::types::{
    CreateCommentRequest, CreateMemoRequest, CreateWebhookRequest, ListMemosParams,
    UpdateMemoRequest,
};

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
pub fn require_name(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "name")
}

/// Build a `CreateMemoRequest` from dispatch params.
pub fn create_request_from(params: &Value) -> Result<CreateMemoRequest, ToolError> {
    let content = require_str(params, "content")?.to_owned();
    let visibility = optional_str(params, "visibility")?.map_or_else(|| "PRIVATE".to_string(), ToOwned::to_owned);
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

/// Extract a required `user` param (user resource name, e.g. `"users/1"`).
pub fn require_user(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "user")
}

/// Build a `CreateCommentRequest` from dispatch params.
pub fn create_comment_request_from(params: &Value) -> Result<CreateCommentRequest, ToolError> {
    let content = require_str(params, "content")?.to_owned();
    Ok(CreateCommentRequest { content })
}

/// Build a `CreateWebhookRequest` from dispatch params.
pub fn create_webhook_request_from(params: &Value) -> Result<CreateWebhookRequest, ToolError> {
    let url = require_str(params, "url")?.to_owned();
    let display_name = require_str(params, "name")?.to_owned();
    Ok(CreateWebhookRequest { url, display_name })
}

/// Extract and decode a `data_base64` param as raw bytes.
///
/// Accepts standard base64, URL-safe base64, with or without padding.
pub fn require_bytes_base64(params: &Value) -> Result<Vec<u8>, ToolError> {
    let b64 = require_str(params, "data_base64")?;
    // Try standard alphabet first, then URL-safe.
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(b64))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(b64))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(b64))
        .map_err(|e| ToolError::InvalidParam {
            message: format!("data_base64 is not valid base64: {e}"),
            param: "data_base64".into(),
        })
}
