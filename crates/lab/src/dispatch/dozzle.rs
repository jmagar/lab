//! Shared dispatch layer for the `dozzle` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::client_from_env;
pub use client::not_configured_error;
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::core::Auth;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn catalog_contains_only_read_only_v1_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"server.health"));
        assert!(names.contains(&"server.version"));
        assert!(names.contains(&"containers.list"));
        assert!(names.contains(&"logs.fetch"));
        assert!(names.contains(&"logs.fetch-plain"));
        assert!(!names.contains(&"events.snapshot"));
        assert!(!names.contains(&"logs.stream-sample"));
    }

    #[test]
    fn all_dozzle_actions_are_non_destructive() {
        assert!(
            ACTIONS.iter().all(|action| !action.destructive),
            "Dozzle v1 must remain read-only"
        );
    }

    #[tokio::test]
    async fn help_lists_primary_action() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(
            actions
                .iter()
                .any(|action| action["name"] == "server.version")
        );
    }

    #[tokio::test]
    async fn dispatch_with_client_round_trips_version() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<pre>v10.5.1</pre>"))
            .mount(&server)
            .await;

        let client = lab_apis::dozzle::DozzleClient::new(&server.uri(), Auth::None).unwrap();
        let value = dispatch_with_client(&client, "server.version", serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(value["version"], "v10.5.1");
    }

    #[tokio::test]
    async fn dispatch_returns_unknown_action_error() {
        let result = dispatch("not.real", serde_json::json!({})).await;
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }
}
