//! Shared dispatch layer for the `gotify` service.

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
    fn catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"message.send"));
        assert!(names.contains(&"message.list"));
        assert!(names.contains(&"app.list"));
        assert!(names.contains(&"client.list"));
        assert!(names.contains(&"server.health"));
    }

    #[test]
    fn destructive_actions_are_marked() {
        let destructive: Vec<&str> = ACTIONS
            .iter()
            .filter(|a| a.destructive)
            .map(|a| a.name)
            .collect();
        assert!(destructive.contains(&"message.delete"));
        assert!(destructive.contains(&"message.purge"));
        assert!(destructive.contains(&"app.delete"));
        assert!(destructive.contains(&"client.delete"));
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
        assert_eq!(val["service"], "gotify");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
    }

}
