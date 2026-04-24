use crate::dispatch::error::ToolError;

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
