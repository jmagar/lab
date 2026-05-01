//! Shared dispatch layer for the `openacp` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::{client_from_env, client_from_instance, not_configured_error};
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ActionRequest, services::helpers::handle_action};
    use lab_apis::core::Auth;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        for name in [
            "help",
            "schema",
            "system.health",
            "system.version",
            "adapters.list",
            "sessions.list",
            "sessions.get",
            "agents.list",
            "config.get",
            "config.editable",
            "topics.list",
            "tunnel.status",
        ] {
            assert!(names.contains(&name), "{name} missing from OpenACP catalog");
        }
    }

    #[test]
    fn all_openacp_actions_are_not_destructive() {
        assert!(
            ACTIONS.iter().all(|action| !action.destructive),
            "OpenACP actions must not use Lab destructive confirmation gates"
        );
    }

    #[tokio::test]
    async fn help_lists_primary_actions() {
        let value = dispatch("help", json!({})).await.unwrap();
        assert_eq!(value["service"], "openacp");
        let actions = value["actions"].as_array().unwrap();
        assert!(
            actions
                .iter()
                .any(|action| action["name"] == "system.health")
        );
        assert!(
            actions
                .iter()
                .any(|action| action["name"] == "sessions.prompt")
        );
    }

    #[tokio::test]
    async fn dispatch_unknown_action_returns_error() {
        let err = dispatch("not.real", json!({})).await.unwrap_err();
        assert_eq!(err.kind(), "unknown_action");
    }

    #[tokio::test]
    async fn missing_session_id_returns_stable_error() {
        let server = MockServer::start().await;
        let client = lab_apis::openacp::OpenAcpClient::new(&server.uri(), Auth::None).unwrap();
        let err = dispatch_with_client(&client, "sessions.get", json!({}))
            .await
            .unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }

    #[tokio::test]
    async fn dispatch_with_client_round_trips_to_sdk_http() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/sessions"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "sessions": []
            })))
            .mount(&server)
            .await;

        let client = lab_apis::openacp::OpenAcpClient::new(
            &server.uri(),
            Auth::Bearer {
                token: "test-token".to_string(),
            },
        )
        .unwrap();
        let value = dispatch_with_client(&client, "sessions.list", json!({}))
            .await
            .unwrap();
        assert_eq!(value["sessions"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn operational_action_is_not_blocked_without_confirm() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/sessions/sess_1/prompt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ok": true,
                "sessionId": "sess_1",
                "queueDepth": 1
            })))
            .mount(&server)
            .await;

        let client = lab_apis::openacp::OpenAcpClient::new(&server.uri(), Auth::None).unwrap();
        let req = ActionRequest {
            action: "sessions.prompt".to_string(),
            params: json!({
                "session_id": "sess_1",
                "prompt": "redacted in logs"
            }),
        };
        let result = handle_action(
            "openacp",
            "api",
            Some("test-request"),
            req,
            ACTIONS,
            |action, params| async move { dispatch_with_client(&client, &action, params).await },
        )
        .await;

        assert!(
            result.is_ok(),
            "OpenACP operational actions must not require params.confirm"
        );
    }
}
