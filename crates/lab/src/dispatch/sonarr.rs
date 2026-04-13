//! Shared dispatch layer for the `Sonarr` service.
//!
//! This module is the single authoritative owner of the Sonarr action catalog,
//! client resolution, and dispatch routing. CLI, MCP, and API are adapters over
//! this surface-neutral layer.

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
    fn catalog_includes_sonarr_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"help"), "missing 'help'");
        assert!(names.contains(&"schema"), "missing 'schema'");
        assert!(names.contains(&"series.list"), "missing 'series.list'");
        assert!(names.contains(&"series.get"), "missing 'series.get'");
        assert!(names.contains(&"series.add"), "missing 'series.add'");
        assert!(names.contains(&"series.delete"), "missing 'series.delete'");
        assert!(names.contains(&"episode.list"), "missing 'episode.list'");
        assert!(names.contains(&"episode.get"), "missing 'episode.get'");
        assert!(names.contains(&"queue.list"), "missing 'queue.list'");
        assert!(names.contains(&"queue.delete"), "missing 'queue.delete'");
        assert!(names.contains(&"history.list"), "missing 'history.list'");
        assert!(names.contains(&"wanted.list"), "missing 'wanted.list'");
        assert!(names.contains(&"health"), "missing 'health'");
        assert!(names.contains(&"system.status"), "missing 'system.status'");
        assert!(names.contains(&"tag.list"), "missing 'tag.list'");
        assert!(names.contains(&"tag.create"), "missing 'tag.create'");
        assert!(names.contains(&"tag.delete"), "missing 'tag.delete'");
        assert!(names.contains(&"rootfolder.list"), "missing 'rootfolder.list'");
        assert!(names.contains(&"qualityprofile.list"), "missing 'qualityprofile.list'");
        assert!(names.contains(&"languageprofile.list"), "missing 'languageprofile.list'");
        assert!(names.contains(&"calendar.list"), "missing 'calendar.list'");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let series_delete = ACTIONS.iter().find(|a| a.name == "series.delete").unwrap();
        assert!(series_delete.destructive, "series.delete must be marked destructive");

        let queue_delete = ACTIONS.iter().find(|a| a.name == "queue.delete").unwrap();
        assert!(queue_delete.destructive, "queue.delete must be marked destructive");

        let tag_delete = ACTIONS.iter().find(|a| a.name == "tag.delete").unwrap();
        assert!(tag_delete.destructive, "tag.delete must be marked destructive");
    }

    #[test]
    fn non_destructive_actions_are_not_marked() {
        let list = ACTIONS.iter().find(|a| a.name == "series.list").unwrap();
        assert!(!list.destructive, "series.list must not be destructive");
        let status = ACTIONS.iter().find(|a| a.name == "system.status").unwrap();
        assert!(!status.destructive, "system.status must not be destructive");
    }

    #[test]
    fn help_lists_series() {
        // Catalog smoke test — no I/O required.
        let help_action = ACTIONS.iter().find(|a| a.name == "help").unwrap();
        assert!(!help_action.destructive);
        assert_eq!(help_action.returns, "Catalog");
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn help_action_returns_catalog() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["service"], "sonarr");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"series.list"));
        assert!(names.contains(&"system.status"));
    }

    #[test]
    fn client_from_env_returns_none_when_not_configured() {
        // Guard: only run if env vars are definitely absent (safe in CI).
        if std::env::var("SONARR_URL").is_err() && std::env::var("SONARR_API_KEY").is_err() {
            assert!(client_from_env().is_none());
        }
    }

    #[test]
    fn no_duplicate_action_names() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            names.len(),
            sorted.len(),
            "duplicate action names found in ACTIONS"
        );
    }
}
