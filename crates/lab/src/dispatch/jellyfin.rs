//! Shared dispatch layer for the `jellyfin` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use client::not_configured_error;
#[allow(unused_imports)]
pub use client::{client_from_env, client_from_instance};
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::core::Auth;
    use lab_apis::jellyfin::JellyfinClient;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client(base_url: &str) -> JellyfinClient {
        JellyfinClient::new(base_url, Auth::None).unwrap()
    }

    #[test]
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        for name in [
            "system.ping",
            "system.info",
            "system.public_info",
            "users.list",
            "users.me",
            "libraries.list",
            "items.search",
            "items.get",
            "items.counts",
            "sessions.list",
            "plugins.list",
        ] {
            assert!(names.contains(&name), "missing {name}");
        }
    }

    #[tokio::test]
    async fn help_lists_first_wave_actions() {
        let value = dispatch("help", json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "items.search"));
    }

    #[tokio::test]
    async fn dispatch_returns_unknown_action_error() {
        let server = MockServer::start().await;
        let err = dispatch_with_client(&test_client(&server.uri()), "not.real", json!({}))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::dispatch::error::ToolError::UnknownAction { .. }
        ));
    }

    #[tokio::test]
    async fn dispatch_with_client_round_trips_items_search() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Items"))
            .and(query_param("searchTerm", "matrix"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Items": [{ "Id": "item-1", "Name": "The Matrix", "Type": "Movie" }],
                "TotalRecordCount": 1,
                "StartIndex": 0
            })))
            .mount(&server)
            .await;

        let value = dispatch_with_client(
            &test_client(&server.uri()),
            "items.search",
            json!({ "search_term": "matrix" }),
        )
        .await
        .unwrap();

        assert_eq!(value["TotalRecordCount"], 1);
        assert_eq!(value["Items"][0]["Name"], "The Matrix");
    }

    #[tokio::test]
    async fn items_get_requires_item_id() {
        let server = MockServer::start().await;
        let err = dispatch_with_client(&test_client(&server.uri()), "items.get", json!({}))
            .await
            .unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }
}
