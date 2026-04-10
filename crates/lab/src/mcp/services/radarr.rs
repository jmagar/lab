//! MCP adapter for the `Radarr` tool.
//!
//! All shared operation logic lives in `crates/lab/src/dispatch/radarr.rs`.
//! This module forwards the catalog and dispatch function for MCP wiring.

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::dispatch::error::ToolError;

pub fn actions() -> &'static [ActionSpec] {
    crate::dispatch::radarr::actions()
}

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    crate::dispatch::radarr::dispatch(action, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_includes_core_read_only_actions() {
        let names: Vec<&str> = actions().iter().map(|a| a.name).collect();
        assert!(names.contains(&"system.status"));
        assert!(names.contains(&"movie.list"));
        assert!(names.contains(&"queue.list"));
        assert!(names.contains(&"calendar.list"));
    }
}
