//! Shared dispatch layer for the `tailscale` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::{client_from_env, not_configured_error, require_client};
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"device.list"));
        assert!(names.contains(&"device.get"));
        assert!(names.contains(&"device.delete"));
        assert!(names.contains(&"key.list"));
        assert!(names.contains(&"dns.nameservers"));
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
        assert!(destructive.contains(&"device.delete"));
        assert!(destructive.contains(&"key.delete"));
        // Non-destructive actions must NOT appear here.
        assert!(!destructive.contains(&"device.list"));
        assert!(!destructive.contains(&"key.list"));
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "unknown_action");
    }

    #[tokio::test]
    async fn help_returns_action_list() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok());
        let val = result.unwrap();
        assert_eq!(val["service"], "tailscale");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
    }
}
