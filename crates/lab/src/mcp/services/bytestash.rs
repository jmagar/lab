//! MCP adapter for the `ByteStash` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/bytestash.rs`.
//! MCP wiring (catalog + dispatch) is done directly in `mcp/registry.rs`.

#[cfg(test)]
mod tests {
    use crate::dispatch::bytestash::ACTIONS;

    #[test]
    fn help_includes_core_read_only_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
        assert!(names.contains(&"categories.list"));
        assert!(names.contains(&"users.list"));
    }
}
