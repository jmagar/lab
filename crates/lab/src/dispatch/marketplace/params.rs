use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

pub(super) struct CherryPickParams {
    pub plugin_id: String,
    pub components: Vec<String>,
    pub node_ids: Vec<String>,
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
    // Defense-in-depth: reject path traversal on component paths even though
    // the device-side handler is expected to validate again. Only `Normal`
    // path components are accepted — no `..`, no absolute paths, no `.` or
    // other special components. Prevents a malicious marketplace manifest
    // or a crafted params payload from inducing an out-of-tree write.
    for component in &components {
        let path = std::path::Path::new(component);
        if path.is_absolute() {
            return Err(ToolError::InvalidParam {
                param: "components".into(),
                message: format!(
                    "component path `{component}` must be relative to the plugin root"
                ),
            });
        }
        for part in path.components() {
            if !matches!(part, std::path::Component::Normal(_)) {
                return Err(ToolError::InvalidParam {
                    param: "components".into(),
                    message: format!(
                        "component path `{component}` contains traversal or invalid segments"
                    ),
                });
            }
        }
    }

    let node_ids: Vec<String> = params
        .get("node_ids")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).map(ToString::to_string).collect())
        .unwrap_or_default();
    if node_ids.is_empty() {
        return Err(ToolError::MissingParam {
            param: "node_ids".into(),
            message: "`node_ids` must be a non-empty array".into(),
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

    Ok(CherryPickParams { plugin_id, components, node_ids, scope, project_path })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base_params() -> Value {
        json!({
            "plugin_id": "demo-plugin@demo-market",
            "components": ["agents/my-agent.md"],
            "node_ids": ["node-1"],
            "scope": "global",
        })
    }

    #[test]
    fn accepts_relative_normal_component_paths() {
        let result = parse_cherry_pick_params(&base_params());
        assert!(result.is_ok(), "valid params must parse: {:?}", result.err());
    }

    #[test]
    fn rejects_component_path_with_parent_dir() {
        let mut params = base_params();
        params["components"] = json!(["agents/../../etc/passwd"]);
        let err = parse_cherry_pick_params(&params).err().expect("must reject");
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn rejects_absolute_component_path() {
        let mut params = base_params();
        params["components"] = json!(["/etc/passwd"]);
        let err = parse_cherry_pick_params(&params).err().expect("must reject");
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn rejects_empty_node_ids() {
        let mut params = base_params();
        params["node_ids"] = json!([]);
        let err = parse_cherry_pick_params(&params).err().expect("must reject");
        assert_eq!(err.kind(), "missing_param");
    }

    #[test]
    fn rejects_relative_project_path() {
        let mut params = base_params();
        params["scope"] = json!("project");
        params["project_path"] = json!("relative/path");
        let err = parse_cherry_pick_params(&params).err().expect("must reject");
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn rejects_project_scope_without_project_path() {
        let mut params = base_params();
        params["scope"] = json!("project");
        let err = parse_cherry_pick_params(&params).err().expect("must reject");
        assert_eq!(err.kind(), "missing_param");
    }
}
