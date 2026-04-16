//! Dispatch layer for the `extract` synthetic service.
//!
//! This is the always-on service; no feature flag needed.
//! All real work is delegated to `lab_apis::extract::ExtractClient`.

use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::extract::{ExtractClient, ScanTarget, Uri};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

/// Action catalog — read by `extract.help`, the `lab.help` meta-tool, and
/// the `lab://extract/actions` MCP resource. **One source of truth**.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        params: &[],
        returns: "Catalog",
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
        returns: "Schema",
    },
    ActionSpec {
        name: "scan",
        description: "Scan an appdata path and return discovered service credentials",
        destructive: false,
        params: &[ParamSpec {
            name: "uri",
            ty: "string",
            required: false,
            description: "Local path or 'host:/abs/path' for SSH; omit for fleet scan",
        }],
        returns: "DiscoveredService[]",
    },
    ActionSpec {
        name: "apply",
        description: "Scan and write discovered credentials into ~/.lab/.env (with backup)",
        destructive: true,
        params: &[
            ParamSpec {
                name: "uri",
                ty: "string",
                required: true,
                description: "Same as scan",
            },
            ParamSpec {
                name: "services",
                ty: "string[]",
                required: false,
                description: "Optional filter; defaults to everything found",
            },
            ParamSpec {
                name: "env_path",
                ty: "string",
                required: false,
                description: "Override target env file path",
            },
        ],
        returns: "WritePlan",
    },
    ActionSpec {
        name: "diff",
        description: "Show what 'apply' would change vs the current env file (no writes)",
        destructive: false,
        params: &[ParamSpec {
            name: "uri",
            ty: "string",
            required: true,
            description: "Local path or 'host:/abs/path' for SSH — same format as scan",
        }],
        returns: "WritePlan",
    },
];

/// Dispatch one call against the extract service.
///
/// # Errors
/// Returns errors from URI parsing, client scan, or unknown action lookup.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("extract", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        "scan" => {
            let target = parse_scan_target(&params)?;
            let client = ExtractClient::new();
            let report = client.scan(target).await.map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: e.to_string(),
            })?;
            serde_json::to_value(report).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: e.to_string(),
            })
        }
        "apply" => {
            // Destructive — the registry has already invoked elicitation
            // before we get here, otherwise dispatch would have short-circuited.
            Err(ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: "apply not yet implemented".into(),
            })
        }
        "diff" => Err(ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: "diff not yet implemented".into(),
        }),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action 'extract.{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

fn parse_uri(params: &Value) -> Result<Uri, ToolError> {
    let s = params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required param 'uri'".into(),
            param: "uri".into(),
        })?;
    s.parse()
        .map_err(|e: <Uri as std::str::FromStr>::Err| ToolError::Sdk {
            sdk_kind: "invalid_param".into(),
            message: e.to_string(),
        })
}

fn parse_scan_target(params: &Value) -> Result<ScanTarget, ToolError> {
    match params.get("uri").and_then(Value::as_str) {
        Some(_uri) => Ok(ScanTarget::Targeted(parse_uri(params)?)),
        None => Ok(ScanTarget::Fleet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::extract::ScanTarget;
    use serde_json::json;

    #[test]
    fn scan_without_uri_maps_to_fleet() {
        let target = parse_scan_target(&json!({})).expect("scan target");
        assert!(matches!(target, ScanTarget::Fleet));
    }

    #[test]
    fn scan_with_uri_maps_to_targeted() {
        let target = parse_scan_target(&json!({"uri": "/tmp/appdata"})).expect("scan target");
        assert!(matches!(target, ScanTarget::Targeted(_)));
    }

    #[test]
    fn apply_and_diff_still_require_uri() {
        assert!(parse_uri(&json!({})).is_err());
    }
}
