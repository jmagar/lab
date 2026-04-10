use serde_json::Value;

use lab_apis::bytestash::types::{AuthCredentials, ShareCreateRequest, SnippetWriteRequest};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::body_from_params;

pub fn credentials_from_params(params: &Value) -> Result<AuthCredentials, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

pub fn snippet_write_from_params(params: &Value) -> Result<SnippetWriteRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

pub fn share_create_from_params(params: &Value) -> Result<ShareCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}
