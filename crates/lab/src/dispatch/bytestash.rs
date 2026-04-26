//! Shared dispatch layer for the `ByteStash` service.
//!
//! Feature-gated: only compiled when the `bytestash` feature is enabled.
//!
//! This module is the single authoritative owner of the `ByteStash` action
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
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
        assert!(names.contains(&"categories.list"));
        assert!(names.contains(&"users.list"));
        assert!(names.contains(&"auth.register"));
    }

    #[test]
    fn auth_register_is_destructive() {
        let spec = ACTIONS.iter().find(|a| a.name == "auth.register").unwrap();
        assert!(spec.destructive, "auth.register must be marked destructive");
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
        assert_eq!(val["service"], "bytestash");
        let actions = val["actions"].as_array().unwrap();
        assert!(!actions.is_empty());
        let names: Vec<&str> = actions.iter().filter_map(|a| a["name"].as_str()).collect();
        assert!(names.contains(&"auth.config"));
        assert!(names.contains(&"snippets.list"));
    }

    #[test]
    fn client_from_vars_rejects_empty_url() {
        assert!(client::client_from_vars(Some(""), Some("tok")).is_none());
    }

    #[test]
    fn client_from_vars_rejects_empty_token() {
        assert!(client::client_from_vars(Some("http://localhost"), Some("")).is_none());
    }

    #[test]
    fn client_from_vars_rejects_missing() {
        assert!(client::client_from_vars(None, Some("tok")).is_none());
        assert!(client::client_from_vars(Some("http://localhost"), None).is_none());
        assert!(client::client_from_vars(None, None).is_none());
    }

    #[test]
    fn client_from_vars_accepts_valid() {
        assert!(
            client::client_from_vars(Some("http://localhost:8080"), Some("jwt-token")).is_some()
        );
    }

    #[test]
    fn params_accept_shared_body_key_for_snippet_writes() {
        let params = serde_json::json!({
            "id": "snippet-1",
            "body": {
                "title": "Shared helper proof",
                "description": "parsed from body",
                "language": "rust",
                "fragments": [],
                "categories": []
            }
        });

        let body = params::snippet_write_from_params(&params).expect("body should parse");
        assert_eq!(body.title, "Shared helper proof");
        assert_eq!(body.language.as_deref(), Some("rust"));
    }
}
