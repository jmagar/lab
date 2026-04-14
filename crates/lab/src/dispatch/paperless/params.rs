use base64::Engine as _;
use serde_json::Value;

use lab_apis::paperless::types::{
    CorrespondentCreateRequest, CustomFieldCreateRequest, DocumentBulkEditRequest,
    DocumentTypeCreateRequest, DocumentUpdateRequest, TagCreateRequest, TagUpdateRequest,
};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{body_from_params, require_i64, require_str};

/// Extract a required `id` param as `u64`, rejecting negative values.
pub fn require_id_u64(params: &Value) -> Result<u64, ToolError> {
    let n = require_i64(params, "id")?;
    u64::try_from(n).map_err(|_| ToolError::InvalidParam {
        message: "parameter `id` must be a non-negative integer".to_string(),
        param: "id".to_string(),
    })
}

/// Build a `DocumentUpdateRequest` from params.
///
/// If a `payload` / `body` key is present its value is used directly (JSON
/// object or JSON-encoded string); otherwise the individual named params are
/// assembled.
pub fn document_update_from_params(params: &Value) -> Result<DocumentUpdateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["id", "payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Build a `TagCreateRequest` from params.
pub fn tag_create_from_params(params: &Value) -> Result<TagCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Build a `CorrespondentCreateRequest` from params.
pub fn correspondent_create_from_params(
    params: &Value,
) -> Result<CorrespondentCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Build a `DocumentTypeCreateRequest` from params.
pub fn document_type_create_from_params(
    params: &Value,
) -> Result<DocumentTypeCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Intermediate struct carrying decoded upload params.
pub struct DocumentUploadParams {
    pub file_bytes: Vec<u8>,
    pub filename: String,
    pub title: Option<String>,
    pub correspondent: Option<u64>,
    pub document_type: Option<u64>,
    pub tags: Option<Vec<u64>>,
}

/// Extract document upload params from a dispatch params value.
pub fn document_upload_from_params(params: &Value) -> Result<DocumentUploadParams, ToolError> {
    let file_b64 = require_str(params, "file_base64")?;
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(file_b64)
        .map_err(|e| ToolError::InvalidParam {
            message: format!("`file_base64` is not valid base64: {e}"),
            param: "file_base64".to_string(),
        })?;
    let filename = require_str(params, "filename")?.to_string();
    let title = params
        .get("title")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let correspondent = params
        .get("correspondent")
        .and_then(Value::as_u64);
    let document_type = params
        .get("document_type")
        .and_then(Value::as_u64);
    let tags = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(Value::as_u64).collect());
    Ok(DocumentUploadParams {
        file_bytes,
        filename,
        title,
        correspondent,
        document_type,
        tags,
    })
}

/// Build a `DocumentBulkEditRequest` from params.
pub fn document_bulk_edit_from_params(params: &Value) -> Result<DocumentBulkEditRequest, ToolError> {
    let documents = params
        .get("documents")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ToolError::MissingParam {
            message: "required parameter `documents` (array of integers) is missing".to_string(),
            param: "documents".to_string(),
        })?
        .iter()
        .filter_map(Value::as_u64)
        .collect();
    let method = require_str(params, "method")?.to_string();
    let parameters = params
        .get("parameters")
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));
    Ok(DocumentBulkEditRequest {
        documents,
        method,
        parameters,
    })
}

/// Build a `TagUpdateRequest` from params (strips `id` before parsing).
pub fn tag_update_from_params(params: &Value) -> Result<TagUpdateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["id", "payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Build a `CustomFieldCreateRequest` from params.
pub fn custom_field_create_from_params(params: &Value) -> Result<CustomFieldCreateRequest, ToolError> {
    serde_json::from_value(body_from_params(params, &["payload", "body"])).map_err(|e| {
        ToolError::InvalidParam {
            message: format!("body parameters could not be parsed: {e}"),
            param: "body".to_string(),
        }
    })
}

/// Extract a `payload` JSON body from params (for free-form endpoints like saved views, storage paths).
pub fn payload_from_params(params: &Value) -> Value {
    // If a `payload` key is present, use it directly; otherwise use the whole object
    // minus well-known meta-keys.
    if let Some(p) = params.get("payload") {
        return p.clone();
    }
    body_from_params(params, &["body"])
}
