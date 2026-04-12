//! Shared dispatch layer for the `SABnzbd` service.
//!
//! This module owns the `SABnzbd` action catalog, client resolution, and
//! dispatch routing. CLI, MCP, and API are adapters over this layer.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use client::client_from_env;
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"queue.list"));
        assert!(names.contains(&"history.list"));
        assert!(names.contains(&"server.version"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let names: Vec<&str> = ACTIONS
            .iter()
            .filter(|a| a.destructive)
            .map(|a| a.name)
            .collect();
        assert!(names.contains(&"queue.delete"));
        assert!(names.contains(&"history.delete"));
        assert!(names.contains(&"history.purge"));
    }
}
