use serde_json::Value;

use lab_apis::paperless::types::{
    CorrespondentCreateRequest, DocumentTypeCreateRequest, DocumentUpdateRequest, TagCreateRequest,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::body_from_params;

/// Build a `DocumentUpdateRequest` from params.
///
/// If a `payload` / `body` key is present its value is used directly (JSON
/// object or JSON-encoded string); otherwise the individual named params are
/// assembled.
pub fn document_update_from_params(params: &Value) -> Result<DocumentUpdateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["id", "payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Build a `TagCreateRequest` from params.
pub fn tag_create_from_params(params: &Value) -> Result<TagCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Build a `CorrespondentCreateRequest` from params.
pub fn correspondent_create_from_params(
    params: &Value,
) -> Result<CorrespondentCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}

/// Build a `DocumentTypeCreateRequest` from params.
pub fn document_type_create_from_params(
    params: &Value,
) -> Result<DocumentTypeCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: e.to_string(),
            param: "payload".to_string(),
        }
    })
}
