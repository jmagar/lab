//! Shared dispatch layer for the `plex` service.
//!
//! This module is the single authoritative owner of the Plex action catalog,
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
    fn catalog_includes_plex_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"help"), "must include help");
        assert!(names.contains(&"schema"), "must include schema");
        assert!(names.contains(&"library.list"), "must include library.list");
        assert!(names.contains(&"media.search"), "must include media.search");
        assert!(names.contains(&"session.list"), "must include session.list");
        assert!(
            names.contains(&"playlist.list"),
            "must include playlist.list"
        );
        assert!(names.contains(&"server.info"), "must include server.info");
        assert!(names.contains(&"health"), "must include health");
    }

    #[test]
    fn destructive_actions_are_marked() {
        let refresh = ACTIONS
            .iter()
            .find(|a| a.name == "library.refresh")
            .unwrap();
        assert!(
            refresh.destructive,
            "library.refresh must be marked destructive"
        );

        let terminate = ACTIONS
            .iter()
            .find(|a| a.name == "session.terminate")
            .unwrap();
        assert!(
            terminate.destructive,
            "session.terminate must be marked destructive"
        );

        let create = ACTIONS
            .iter()
            .find(|a| a.name == "playlist.create")
            .unwrap();
        assert!(
            create.destructive,
            "playlist.create must be marked destructive"
        );

        let delete = ACTIONS
            .iter()
            .find(|a| a.name == "playlist.delete")
            .unwrap();
        assert!(
            delete.destructive,
            "playlist.delete must be marked destructive"
        );
    }

    #[test]
    fn non_destructive_actions_are_not_marked() {
        let list = ACTIONS.iter().find(|a| a.name == "library.list").unwrap();
        assert!(!list.destructive, "library.list must not be destructive");

        let search = ACTIONS.iter().find(|a| a.name == "media.search").unwrap();
        assert!(!search.destructive, "media.search must not be destructive");

        let sessions = ACTIONS.iter().find(|a| a.name == "session.list").unwrap();
        assert!(
            !sessions.destructive,
            "session.list must not be destructive"
        );
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

    #[tokio::test]
    async fn help_lists_library() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok(), "help must succeed");
        let val = result.unwrap();
        assert_eq!(val["service"], "plex");
        let actions = val["actions"].as_array().unwrap();
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"library.list"));
        assert!(names.contains(&"media.search"));
        assert!(names.contains(&"session.list"));
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn schema_returns_spec_for_known_action() {
        let result = dispatch("schema", serde_json::json!({ "action": "media.search" })).await;
        assert!(result.is_ok(), "schema for media.search must succeed");
        let val = result.unwrap();
        assert_eq!(val["action"], "media.search");
    }
}
