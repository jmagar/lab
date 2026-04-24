use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

pub(super) struct CherryPickParams {
    pub plugin_id: String,
    pub components: Vec<String>,
    pub device_ids: Vec<String>,
    pub scope: String,
    pub project_path: Option<String>,
}

pub(super) fn parse_cherry_pick_params(params: &Value) -> Result<CherryPickParams, ToolError> {
    let plugin_id = require_str(params, "plugin_id")?.to_string();
    parse_plugin_id(&plugin_id)?;

    let components: Vec<String> = params
        .get("components")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).map(ToString::to_string).collect())
        .unwrap_or_default();
    if components.is_empty() {
        return Err(ToolError::MissingParam {
            param: "components".into(),
            message: "`components` must be a non-empty array".into(),
        });
    }

    let device_ids: Vec<String> = params
        .get("device_ids")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).map(ToString::to_string).collect())
        .unwrap_or_default();
    if device_ids.is_empty() {
        return Err(ToolError::MissingParam {
            param: "device_ids".into(),
            message: "`device_ids` must be a non-empty array".into(),
        });
    }

    let scope = require_str(params, "scope")?.to_string();
    if scope != "global" && scope != "project" {
        return Err(ToolError::InvalidParam {
            param: "scope".into(),
            message: "`scope` must be `global` or `project`".into(),
        });
    }

    let project_path = params.get("project_path").and_then(Value::as_str).map(ToString::to_string);
    if scope == "project" {
        match &project_path {
            None => {
                return Err(ToolError::MissingParam {
                    param: "project_path".into(),
                    message: "`project_path` is required when `scope` is `project`".into(),
                });
            }
            Some(p) if !p.starts_with('/') => {
                return Err(ToolError::InvalidParam {
                    param: "project_path".into(),
                    message: "`project_path` must be an absolute path".into(),
                });
            }
            _ => {}
        }
    }

    Ok(CherryPickParams { plugin_id, components, device_ids, scope, project_path })
}

/// Parse a plugin id in `name@marketplace` form.
///
/// Both components are validated against path traversal: only `Normal` path
/// components are accepted, rejecting `..`, absolute paths, and `.`.
pub fn parse_plugin_id(id: &str) -> Result<(&str, &str), ToolError> {
    let (name, marketplace) = id
        .split_once('@')
        .filter(|(n, m)| !n.is_empty() && !m.is_empty() && !m.contains('@'))
        .ok_or_else(|| ToolError::InvalidParam {
            message: format!("plugin id `{id}` must be in `name@marketplace` form"),
            param: "id".into(),
        })?;
    for part in [name, marketplace] {
        for component in std::path::Path::new(part).components() {
            if !matches!(component, std::path::Component::Normal(_)) {
                return Err(ToolError::InvalidParam {
                    message: format!("plugin id `{id}` contains invalid path characters"),
                    param: "id".into(),
                });
            }
        }
    }
    Ok((name, marketplace))
}
