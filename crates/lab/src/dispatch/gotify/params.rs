use serde_json::Value;

use lab_apis::gotify::types::{
    ApplicationId, ApplicationParams, ClientId, ClientParams, MessageId, PluginId, SendMessage,
    UserCreate, UserId,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Build a `SendMessage` from action params.
pub fn send_message_from_params(params: &Value) -> Result<SendMessage, ToolError> {
    let message = require_str(params, "message")?.to_string();
    Ok(SendMessage {
        message,
        title: params
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        priority: priority_from_params(params)?,
        extras: None,
    })
}

fn priority_from_params(params: &Value) -> Result<Option<i32>, ToolError> {
    match params.get("priority") {
        None => Ok(None),
        Some(v) => {
            let priority = v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `priority` must be an integer".to_string(),
                param: "priority".to_string(),
            })?;
            let priority = i32::try_from(priority).map_err(|_| ToolError::InvalidParam {
                message: "parameter `priority` must be between 0 and 10".to_string(),
                param: "priority".to_string(),
            })?;
            if !(0..=10).contains(&priority) {
                return Err(ToolError::InvalidParam {
                    message: "parameter `priority` must be between 0 and 10".to_string(),
                    param: "priority".to_string(),
                });
            }
            Ok(Some(priority))
        }
    }
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

/// Extract a `PluginId` from the `id` param.
pub fn plugin_id_from_params(params: &Value) -> Result<PluginId, ToolError> {
    params
        .get("id")
        .and_then(Value::as_u64)
        .map(PluginId)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `id`".to_string(),
            param: "id".to_string(),
        })
}

/// Extract a `UserId` from the `id` param.
pub fn user_id_from_params(params: &Value) -> Result<UserId, ToolError> {
    params
        .get("id")
        .and_then(Value::as_u64)
        .map(UserId)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `id`".to_string(),
            param: "id".to_string(),
        })
}

/// Build a `UserCreate` from action params.
pub fn user_create_from_params(params: &Value) -> Result<UserCreate, ToolError> {
    let name = require_str(params, "name")?.to_string();
    let pass = require_str(params, "pass")?.to_string();
    let admin = params
        .get("admin")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(UserCreate { name, pass, admin })
}
