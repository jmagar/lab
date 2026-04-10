use serde_json::Value;

use lab_apis::gotify::types::{
    ApplicationId, ApplicationParams, ClientId, ClientParams, MessageId, SendMessage,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Build a `SendMessage` from action params.
pub fn send_message_from_params(params: &Value) -> Result<SendMessage, ToolError> {
    let message = require_str(params, "message")?.to_string();
    Ok(SendMessage {
        message,
        title: params.get("title").and_then(Value::as_str).map(str::to_string),
        priority: params.get("priority").and_then(Value::as_i64).map(|n| i32::try_from(n).unwrap_or(i32::MAX)),
        extras: None,
    })
}

/// Extract a `MessageId` from the `id` param.
pub fn message_id_from_params(params: &Value) -> Result<MessageId, ToolError> {
    params
        .get("id")
        .and_then(Value::as_u64)
        .map(MessageId)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `id`".to_string(),
            param: "id".to_string(),
        })
}

/// Extract an `ApplicationId` from the `id` param.
pub fn application_id_from_params(params: &Value) -> Result<ApplicationId, ToolError> {
    params
        .get("id")
        .and_then(Value::as_u64)
        .map(ApplicationId)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `id`".to_string(),
            param: "id".to_string(),
        })
}

/// Build an `ApplicationParams` from action params.
pub fn application_params_from_params(params: &Value) -> Result<ApplicationParams, ToolError> {
    let name = require_str(params, "name")?.to_string();
    Ok(ApplicationParams {
        name,
        description: params
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// Extract a `ClientId` from the `id` param.
pub fn client_id_from_params(params: &Value) -> Result<ClientId, ToolError> {
    params
        .get("id")
        .and_then(Value::as_u64)
        .map(ClientId)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `id`".to_string(),
            param: "id".to_string(),
        })
}

/// Build a `ClientParams` from action params.
pub fn client_params_from_params(params: &Value) -> Result<ClientParams, ToolError> {
    let name = require_str(params, "name")?.to_string();
    Ok(ClientParams { name })
}
