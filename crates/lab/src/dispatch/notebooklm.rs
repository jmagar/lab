//! Shared dispatch layer for the `notebooklm` service.

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
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"help"));
        assert!(names.contains(&"schema"));
        assert!(names.contains(&"notebook.list"));
        assert!(names.contains(&"notebook.create"));
        assert!(names.contains(&"source.add_url"));
        assert!(names.contains(&"server.health"));
    }

    #[test]
    fn delete_notebook_is_destructive() {
        let action = ACTIONS
            .iter()
            .find(|a| a.name == "notebook.delete")
            .expect("delete action");
        assert!(action.destructive);
    }

    #[tokio::test]
    async fn help_lists_notebook_list() {
        let result = dispatch("help", serde_json::json!({})).await.unwrap();
        assert_eq!(result["service"], "notebooklm");
        let actions = result["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "notebook.list"));
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "unknown_action");
    }
}
