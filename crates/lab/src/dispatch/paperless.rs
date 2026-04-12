//! Shared dispatch layer for the `Paperless-ngx` service.
//!
//! Feature-gated: only compiled when the `paperless` feature is enabled.
//!
//! This module is the single authoritative owner of the Paperless-ngx action
//! catalog, client resolution, and shared dispatch semantics.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
// Re-exported for AppState wiring and API surface use.
#[allow(unused_imports)]
pub use client::client_from_env;
pub use dispatch::dispatch;
#[allow(unused_imports)]
pub use dispatch::dispatch_with_client;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"documents.list"));
        assert!(names.contains(&"documents.get"));
        assert!(names.contains(&"documents.update"));
        assert!(names.contains(&"documents.delete"));
        assert!(names.contains(&"tags.list"));
        assert!(names.contains(&"tags.create"));
        assert!(names.contains(&"correspondents.list"));
        assert!(names.contains(&"correspondents.create"));
        assert!(names.contains(&"document_types.list"));
        assert!(names.contains(&"document_types.create"));
        assert!(names.contains(&"statistics"));
        assert!(names.contains(&"tasks.list"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let check = |name: &str| {
            let spec = ACTIONS.iter().find(|a| a.name == name).unwrap_or_else(|| {
                panic!("action '{name}' not found in catalog")
            });
            assert!(spec.destructive, "action '{name}' must be marked destructive");
        };
        check("documents.update");
        check("documents.delete");
        check("tags.create");
        check("tags.delete");
        check("correspondents.create");
        check("correspondents.delete");
        check("document_types.create");
        check("document_types.delete");
    }

    #[test]
    fn read_actions_are_not_destructive() {
        let check = |name: &str| {
            let spec = ACTIONS.iter().find(|a| a.name == name).unwrap_or_else(|| {
                panic!("action '{name}' not found in catalog")
            });
            assert!(
                !spec.destructive,
                "action '{name}' must NOT be marked destructive"
            );
        };
        check("documents.list");
        check("documents.get");
        check("documents.metadata");
        check("tags.list");
        check("tags.get");
        check("correspondents.list");
        check("correspondents.get");
        check("document_types.list");
        check("document_types.get");
        check("statistics");
        check("tasks.list");
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
        assert_eq!(val["service"], "paperless");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions
            .iter()
            .filter_map(|a| a["name"].as_str())
            .collect();
        assert!(names.contains(&"documents.list"));
        assert!(names.contains(&"statistics"));
    }

    #[test]
    fn client_from_env_rejects_missing_vars() {
        // No env vars set in test env — should return None.
        // (If the test runner happens to have PAPERLESS_URL set this test
        // is still correct because we only assert None when both are absent.)
        if std::env::var("PAPERLESS_URL").is_err() && std::env::var("PAPERLESS_TOKEN").is_err() {
            assert!(client_from_env().is_none());
        }
    }
}
