//! MCP adapter for the `mcpregistry` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/mcpregistry`.
//! MCP wiring (catalog + dispatch) is done directly in `mcp/registry.rs`.

#[cfg(test)]
mod tests {
    use crate::dispatch::mcpregistry::ACTIONS;

    #[test]
    fn catalog_includes_read_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"server.list"));
        assert!(names.contains(&"server.get"));
        assert!(names.contains(&"server.versions"));
    }
}
