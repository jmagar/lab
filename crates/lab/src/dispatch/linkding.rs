//! Shared dispatch layer for the `Linkding` service.
//!
//! Feature-gated: only compiled when the `linkding` feature is enabled.
//!
//! This module is the single authoritative owner of the `Linkding` action
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
        assert!(names.contains(&"bookmark.list"));
        assert!(names.contains(&"bookmark.archived.list"));
        assert!(names.contains(&"bookmark.get"));
        assert!(names.contains(&"bookmark.check"));
        assert!(names.contains(&"bookmark.create"));
        assert!(names.contains(&"bookmark.update"));
        assert!(names.contains(&"bookmark.archive"));
        assert!(names.contains(&"bookmark.unarchive"));
        assert!(names.contains(&"bookmark.delete"));
        assert!(names.contains(&"tag.list"));
        assert!(names.contains(&"tag.get"));
        assert!(names.contains(&"tag.create"));
        assert!(names.contains(&"user.profile"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let delete = ACTIONS
            .iter()
            .find(|a| a.name == "bookmark.delete")
            .unwrap();
        assert!(
            delete.destructive,
            "bookmark.delete must be marked destructive"
        );
    }

    #[test]
    fn non_destructive_actions_are_not_marked() {
        let list = ACTIONS.iter().find(|a| a.name == "bookmark.list").unwrap();
        assert!(
            !list.destructive,
            "bookmark.list must not be marked destructive"
        );

        let create = ACTIONS
            .iter()
            .find(|a| a.name == "bookmark.create")
            .unwrap();
        assert!(
            !create.destructive,
            "bookmark.create must not be marked destructive"
        );

        let archive = ACTIONS
            .iter()
            .find(|a| a.name == "bookmark.archive")
            .unwrap();
        assert!(
            !archive.destructive,
            "bookmark.archive must not be marked destructive"
        );
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
        assert_eq!(val["service"], "linkding");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"bookmark.list"));
        assert!(names.contains(&"tag.list"));
        assert!(names.contains(&"user.profile"));
    }

    #[test]
    fn client_from_vars_rejects_empty_url() {
        assert!(client::client_from_vars(Some(""), Some("token123")).is_none());
    }

    #[test]
    fn client_from_vars_rejects_empty_token() {
        assert!(client::client_from_vars(Some("http://localhost"), Some("")).is_none());
    }

    #[test]
    fn client_from_vars_rejects_missing() {
        assert!(client::client_from_vars(None, Some("token123")).is_none());
        assert!(client::client_from_vars(Some("http://localhost"), None).is_none());
        assert!(client::client_from_vars(None, None).is_none());
    }

    #[test]
    fn client_from_vars_accepts_valid() {
        assert!(
            client::client_from_vars(Some("http://localhost:9090"), Some("mytoken123")).is_some()
        );
    }
}
