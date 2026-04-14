//! Shared dispatch layer for the `Prowlarr` service.
//!
//! Feature-gated: only compiled when the `prowlarr` feature is enabled.
//!
//! This module is the single authoritative owner of the `Prowlarr` action
//! catalog, client resolution, and shared dispatch semantics.

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
        assert!(names.contains(&"indexer.list"));
        assert!(names.contains(&"indexer.get"));
        assert!(names.contains(&"indexer.delete"));
        assert!(names.contains(&"indexer.test"));
        assert!(names.contains(&"indexer.testall"));
        assert!(names.contains(&"indexer.categories"));
        assert!(names.contains(&"history.list"));
        assert!(names.contains(&"application.list"));
        assert!(names.contains(&"application.get"));
        assert!(names.contains(&"application.delete"));
        assert!(names.contains(&"system.status"));
        assert!(names.contains(&"system.health"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let indexer_delete = ACTIONS.iter().find(|a| a.name == "indexer.delete").unwrap();
        assert!(
            indexer_delete.destructive,
            "indexer.delete must be marked destructive"
        );
        let app_delete = ACTIONS
            .iter()
            .find(|a| a.name == "application.delete")
            .unwrap();
        assert!(
            app_delete.destructive,
            "application.delete must be marked destructive"
        );
    }

    #[test]
    fn non_destructive_actions_are_not_marked() {
        let list = ACTIONS.iter().find(|a| a.name == "indexer.list").unwrap();
        assert!(!list.destructive, "indexer.list must not be destructive");
        let status = ACTIONS.iter().find(|a| a.name == "system.status").unwrap();
        assert!(!status.destructive, "system.status must not be destructive");
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn help_returns_action_list() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["service"], "prowlarr");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"indexer.list"));
        assert!(names.contains(&"system.status"));
    }

    #[test]
    fn client_from_env_returns_none_when_not_configured() {
        // Guard: only run if env vars are definitely absent (safe in CI).
        if std::env::var("PROWLARR_URL").is_err() && std::env::var("PROWLARR_API_KEY").is_err() {
            assert!(client_from_env().is_none());
        }
    }
}
