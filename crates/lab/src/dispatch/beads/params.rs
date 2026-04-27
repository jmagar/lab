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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn no_params_returns_defaults() {
        let p = issue_list_params(&json!({})).unwrap();
        assert!(p.status.is_none());
        assert!(p.limit.is_none());
        assert!(p.offset.is_none());
    }

    #[test]
    fn valid_limit_and_offset_accepted() {
        let p = issue_list_params(&json!({"limit": 10, "offset": 5})).unwrap();
        assert_eq!(p.limit, Some(10));
        assert_eq!(p.offset, Some(5));
    }

    #[test]
    fn zero_limit_accepted() {
        // Zero is technically non-negative; clamping to ≥1 happens in the client layer.
        let p = issue_list_params(&json!({"limit": 0})).unwrap();
        assert_eq!(p.limit, Some(0));
    }

    #[test]
    fn negative_limit_rejected() {
        let err = issue_list_params(&json!({"limit": -1})).unwrap_err();
        match err {
            ToolError::InvalidParam { param, .. } => assert_eq!(param, "limit"),
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn negative_offset_rejected() {
        let err = issue_list_params(&json!({"offset": -5})).unwrap_err();
        match err {
            ToolError::InvalidParam { param, .. } => assert_eq!(param, "offset"),
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn non_integer_limit_rejected() {
        let err = issue_list_params(&json!({"limit": "big"})).unwrap_err();
        match err {
            ToolError::InvalidParam { param, .. } => assert_eq!(param, "limit"),
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn filters_are_passed_through() {
        let p = issue_list_params(&json!({
            "status": "open",
            "issue_type": "bug",
            "owner": "alice",
            "label": "p1"
        }))
        .unwrap();
        assert_eq!(p.status.as_deref(), Some("open"));
        assert_eq!(p.issue_type.as_deref(), Some("bug"));
        assert_eq!(p.owner.as_deref(), Some("alice"));
        assert_eq!(p.label.as_deref(), Some("p1"));
    }
}
