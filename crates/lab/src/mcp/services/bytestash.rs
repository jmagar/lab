//! MCP adapter for the `ByteStash` tool.
//!
//! All shared operation logic lives in `crates/lab/src/services/bytestash.rs`.
//! This module re-exports the catalog and dispatch function for MCP wiring.

pub use crate::services::bytestash::{ACTIONS, dispatch};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_includes_core_read_only_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
        assert!(names.contains(&"categories.list"));
        assert!(names.contains(&"users.list"));
    }
}
