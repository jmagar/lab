//! Shared dispatch layer for the `unraid` service.
//!
//! Feature-gated: only compiled when the `unraid` feature is enabled.
//!
//! This module is the single authoritative owner of the Unraid action
//! catalog, client resolution, and shared dispatch semantics.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use client::{client_from_env, client_from_instance, not_configured_error};
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"system.info"));
        assert!(names.contains(&"system.metrics"));
        assert!(names.contains(&"system.array"));
        assert!(names.contains(&"system.online"));
        assert!(names.contains(&"docker.list"));
        assert!(names.contains(&"docker.start"));
        assert!(names.contains(&"docker.stop"));
        assert!(names.contains(&"docker.restart"));
        assert!(names.contains(&"disk.list"));
    }

    #[test]
    fn docker_mutations_are_destructive() {
        for name in &["docker.start", "docker.stop", "docker.restart"] {
            let spec = ACTIONS.iter().find(|a| a.name == *name).unwrap();
            assert!(spec.destructive, "{name} must be marked destructive");
        }
    }

    #[test]
    fn read_only_actions_are_not_destructive() {
        for name in &[
            "system.info",
            "system.metrics",
            "system.array",
            "system.online",
            "docker.list",
            "disk.list",
        ] {
            let spec = ACTIONS.iter().find(|a| a.name == *name).unwrap();
            assert!(!spec.destructive, "{name} must not be marked destructive");
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
        assert_eq!(val["service"], "unraid");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"system.info"));
        assert!(names.contains(&"docker.list"));
    }
}
