//! Shared dispatch layer for the `apprise` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::client_from_env;
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::core::Auth;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"server.health"));
        assert!(names.contains(&"notify.send"));
        assert!(names.contains(&"notify.key.send"));
    }

    #[tokio::test]
    async fn help_lists_notify_send() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "notify.send"));
    }

    #[tokio::test]
    async fn dispatch_with_client_sends_notification() {
        let server = MockServer::start().await;
        let params = serde_json::json!({
            "urls": ["json://localhost"],
            "title": "Test",
            "body": "hello"
        });
        Mock::given(method("POST"))
            .and(path("/notify"))
            .and(body_json(params.clone()))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = lab_apis::apprise::AppriseClient::new(&server.uri(), Auth::None).unwrap();
        let value = dispatch_with_client(&client, "notify.send", params)
            .await
            .unwrap();

        assert_eq!(value, serde_json::Value::Null);
    }
}
