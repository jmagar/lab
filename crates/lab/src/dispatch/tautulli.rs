//! Shared dispatch layer for the `Tautulli` service.
//!
//! Feature-gated: only compiled when the `tautulli` feature is enabled.
//!
//! This module is the single authoritative owner of the `Tautulli` action
//! catalog, client resolution, and shared dispatch semantics.

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
    fn actions_catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"activity.list"));
        assert!(names.contains(&"history.list"));
        assert!(names.contains(&"users.list"));
        assert!(names.contains(&"libraries.list"));
        assert!(names.contains(&"stats.home"));
        assert!(names.contains(&"system.info"));
        assert!(names.contains(&"help"));
        assert!(names.contains(&"schema"));
    }

    #[test]
    fn no_actions_are_destructive() {
        // Tautulli is a read-only analytics tool — no destructive actions.
        for action in ACTIONS {
            assert!(
                !action.destructive,
                "action '{}' must not be marked destructive (Tautulli is read-only)",
                action.name
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
        assert_eq!(val["service"], "tautulli");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"activity.list"));
        assert!(names.contains(&"history.list"));
        assert!(names.contains(&"system.info"));
    }

    #[test]
    fn client_from_env_returns_none_when_not_configured() {
        // Guard: only run if env vars are definitely absent (safe in CI).
        if std::env::var("TAUTULLI_URL").is_err() && std::env::var("TAUTULLI_API_KEY").is_err() {
            assert!(client_from_env().is_none());
        }
    }
}
