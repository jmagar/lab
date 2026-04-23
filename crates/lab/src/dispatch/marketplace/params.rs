use crate::dispatch::error::ToolError;

/// Parse a plugin id in `name@marketplace` form.
pub fn parse_plugin_id(id: &str) -> Result<(&str, &str), ToolError> {
    id.split_once('@')
        .filter(|(n, m)| !n.is_empty() && !m.is_empty() && !m.contains('@'))
        .ok_or_else(|| ToolError::InvalidParam {
            message: format!("plugin id `{id}` must be in `name@marketplace` form"),
            param: "id".into(),
        })
}
