//! Shared dispatch layer for the `mcpregistry` service.
//!
//! Feature-gated: only compiled when the `mcpregistry` feature is enabled.

mod catalog;
pub mod client;
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
        assert!(names.contains(&"server.list"));
        assert!(names.contains(&"server.get"));
        assert!(names.contains(&"server.versions"));
        assert!(names.contains(&"help"));
        assert!(names.contains(&"schema"));
    }

    #[test]
    fn no_action_is_destructive() {
        for spec in ACTIONS {
            assert!(
                !spec.destructive,
                "action '{}' must not be destructive — mcpregistry is read-only",
                spec.name
            );
        }
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
        assert_eq!(val["service"], "mcpregistry");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"server.list"));
        assert!(names.contains(&"server.get"));
    }
}
