//! Parameter extraction for beads dispatch.

use lab_apis::beads::types::IssueListParams;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, require_str};

/// Extract `IssueListParams` from a JSON params object.
///
/// All filters are optional. `limit` and `offset` are coerced to `i64` and
/// rejected when negative or non-numeric.
pub fn issue_list_params(params: &Value) -> Result<IssueListParams, ToolError> {
    let status = optional_str(params, "status")?.map(ToOwned::to_owned);
    let issue_type = optional_str(params, "issue_type")?.map(ToOwned::to_owned);
    let owner = optional_str(params, "owner")?.map(ToOwned::to_owned);
    let label = optional_str(params, "label")?.map(ToOwned::to_owned);

    let limit = match params.get("limit") {
        None => None,
        Some(v) => {
            let n = v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `limit` must be an integer".to_string(),
                param: "limit".to_string(),
            })?;
            if n < 0 {
                return Err(ToolError::InvalidParam {
                    message: "parameter `limit` must not be negative".to_string(),
                    param: "limit".to_string(),
                });
            }
            Some(n)
        }
    };
    let offset = match params.get("offset") {
        None => None,
        Some(v) => {
            let n = v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: "parameter `offset` must be an integer".to_string(),
                param: "offset".to_string(),
            })?;
            if n < 0 {
                return Err(ToolError::InvalidParam {
                    message: "parameter `offset` must not be negative".to_string(),
                    param: "offset".to_string(),
                });
            }
            Some(n)
        }
    };

    Ok(IssueListParams {
        status,
        issue_type,
        owner,
        label,
        limit,
        offset,
    })
}

/// Extract `id` from a JSON params object.
pub fn require_id<'a>(params: &'a Value) -> Result<&'a str, ToolError> {
    require_str(params, "id")
}
