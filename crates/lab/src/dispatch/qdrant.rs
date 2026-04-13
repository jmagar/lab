//! Shared dispatch layer for the `qdrant` service.

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
        assert!(names.contains(&"collections.list"));
        assert!(names.contains(&"collections.get"));
    }

    #[tokio::test]
    async fn help_lists_collections_list() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "collections.list"));
    }

    #[tokio::test]
    async fn dispatch_with_client_lists_collections() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/collections"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "time": 0.001,
                "status": "ok",
                "result": {
                    "collections": [
                        { "name": "axon" },
                        { "name": "mem0" }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let client = lab_apis::qdrant::QdrantClient::new(&server.uri(), Auth::None).unwrap();
        let value = dispatch_with_client(&client, "collections.list", serde_json::json!({}))
            .await
            .unwrap();

        let collections = value.as_array().unwrap();
        assert_eq!(collections.len(), 2);
        assert_eq!(collections[0]["name"], "axon");
        assert_eq!(collections[1]["name"], "mem0");
    }
}
