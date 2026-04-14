//! Shared dispatch layer for the `Overseerr` service.
//!
//! This module is the single authoritative owner of the Overseerr action
//! catalog, client resolution, and dispatch routing. CLI, MCP, and API are
//! thin adapters over this surface-neutral layer.

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
    fn catalog_includes_overseerr_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"help"), "help must be in catalog");
        assert!(names.contains(&"schema"), "schema must be in catalog");
        assert!(names.contains(&"request.list"));
        assert!(names.contains(&"request.get"));
        assert!(names.contains(&"request.create"));
        assert!(names.contains(&"request.approve"));
        assert!(names.contains(&"request.decline"));
        assert!(names.contains(&"request.delete"));
        assert!(names.contains(&"movie.search"));
        assert!(names.contains(&"tv.search"));
        assert!(names.contains(&"movie.get"));
        assert!(names.contains(&"tv.get"));
        assert!(names.contains(&"user.list"));
        assert!(names.contains(&"user.get"));
        assert!(names.contains(&"issue.list"));
        assert!(names.contains(&"issue.get"));
        assert!(names.contains(&"issue.create"));
        assert!(names.contains(&"issue.comment"));
        assert!(names.contains(&"health"));
        assert!(names.contains(&"status"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let delete = ACTIONS.iter().find(|a| a.name == "request.delete").unwrap();
        assert!(
            delete.destructive,
            "request.delete must be marked destructive"
        );
    }

    #[test]
    fn non_destructive_actions_are_not_marked() {
        let list = ACTIONS.iter().find(|a| a.name == "request.list").unwrap();
        assert!(!list.destructive);

        let search = ACTIONS.iter().find(|a| a.name == "movie.search").unwrap();
        assert!(!search.destructive);

        let status = ACTIONS.iter().find(|a| a.name == "status").unwrap();
        assert!(!status.destructive);

        let health = ACTIONS.iter().find(|a| a.name == "health").unwrap();
        assert!(!health.destructive);
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
    async fn help_lists_request_actions() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok(), "help should succeed");
        let val = result.unwrap();
        assert_eq!(val["service"], "overseerr");
        let actions = val["actions"].as_array().unwrap();
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"request.list"));
        assert!(names.contains(&"movie.search"));
        assert!(names.contains(&"health"));
        assert!(names.contains(&"status"));
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn schema_returns_action_schema() {
        let result = dispatch("schema", serde_json::json!({ "action": "request.list" })).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["action"], "request.list");
        assert!(val["params"].is_array());
    }

    #[tokio::test]
    async fn schema_unknown_action_returns_error() {
        let result = dispatch("schema", serde_json::json!({ "action": "bogus" })).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "unknown_action");
    }
}
