//! Shared dispatch layer for the `openai` service.

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
    fn catalog_includes_core_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"help"));
        assert!(names.contains(&"schema"));
        assert!(names.contains(&"model.list"));
        assert!(names.contains(&"chat.complete"));
        assert!(names.contains(&"embed.create"));
        assert!(names.contains(&"server.health"));
    }

    #[test]
    fn no_destructive_actions() {
        let destructive: Vec<&str> = ACTIONS
            .iter()
            .filter(|a| a.destructive)
            .map(|a| a.name)
            .collect();
        assert!(
            destructive.is_empty(),
            "unexpected destructive actions: {destructive:?}"
        );
    }

    #[tokio::test]
    async fn help_lists_model_list() {
        let result = dispatch("help", serde_json::json!({})).await;
        assert!(result.is_ok(), "help dispatch failed: {result:?}");
        let val = result.unwrap();
        assert_eq!(val["service"], "openai");
        let actions = val["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "model.list"));
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let result = dispatch("not.a.real.action", serde_json::json!({})).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "unknown_action");
    }

    #[tokio::test]
    async fn dispatch_with_client_model_list() {
        use lab_apis::core::Auth;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "list",
                "data": [
                    {
                        "id": "gpt-4o-mini",
                        "object": "model",
                        "created": 1706000000,
                        "owned_by": "openai"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = lab_apis::openai::OpenAiClient::new(
            &server.uri(),
            Auth::Bearer {
                token: "test-key".into(),
            },
        )
        .unwrap();
        let result = dispatch_with_client(&client, "model.list", serde_json::json!({}))
            .await
            .unwrap();

        let data = result["data"].as_array().unwrap();
        assert!(!data.is_empty());
        assert_eq!(data[0]["id"], "gpt-4o-mini");
    }
}
