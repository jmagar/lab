//! Shared dispatch layer for the `tei` service.

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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"server.health"));
        assert!(names.contains(&"server.info"));
        assert!(names.contains(&"embed.create"));
    }

    #[tokio::test]
    async fn help_lists_server_health() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "server.health"));
    }

    #[tokio::test]
    async fn dispatch_with_client_returns_model_info() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model_id": "test-model",
                "version": "1.2.3"
            })))
            .mount(&server)
            .await;

        let client = lab_apis::tei::TeiClient::new(&server.uri(), Auth::None).unwrap();
        let value = dispatch_with_client(&client, "server.info", serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(value["model_id"], "test-model");
        assert_eq!(value["version"], "1.2.3");
    }
}
