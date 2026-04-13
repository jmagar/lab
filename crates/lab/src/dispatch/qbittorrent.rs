//! Shared dispatch layer for the `qbittorrent` service.
//!
//! This module owns the qBittorrent action catalog, client resolution, and
//! dispatch routing. CLI, MCP, and API are adapters over this layer.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::{client_from_env, not_configured_error};
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_includes_key_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"torrent.list"));
        assert!(names.contains(&"transfer.info"));
        assert!(names.contains(&"app.version"));
        assert!(names.contains(&"help"));
        assert!(names.contains(&"schema"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let destructive: Vec<&str> = ACTIONS
            .iter()
            .filter(|a| a.destructive)
            .map(|a| a.name)
            .collect();
        assert!(destructive.contains(&"torrent.delete"));
    }

    #[test]
    fn non_destructive_pause_and_resume() {
        let spec_pause = ACTIONS.iter().find(|a| a.name == "torrent.pause").unwrap();
        let spec_resume = ACTIONS.iter().find(|a| a.name == "torrent.resume").unwrap();
        assert!(!spec_pause.destructive);
        assert!(!spec_resume.destructive);
    }
}
