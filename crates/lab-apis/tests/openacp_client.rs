#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::openacp::OpenAcpClient;
use lab_apis::openacp::types::PromptRequest;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

fn client(base_url: &str, auth: Auth) -> OpenAcpClient {
    OpenAcpClient::new(base_url, auth).expect("client construction")
}

#[tokio::test]
async fn health_and_version_do_not_require_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "version": "0.6.7"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "0.6.7"
        })))
        .mount(&server)
        .await;

    let client = client(&server.uri(), Auth::None);
    assert_eq!(client.health().await.unwrap()["status"], "ok");
    assert_eq!(client.version().await.unwrap()["version"], "0.6.7");
}

#[tokio::test]
async fn protected_calls_send_bearer_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/agents"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "agents": [],
            "default": "claude"
        })))
        .mount(&server)
        .await;

    let client = client(
        &server.uri(),
        Auth::Bearer {
            token: "test-token".to_string(),
        },
    );
    assert_eq!(client.agents_list().await.unwrap()["default"], "claude");
}

#[tokio::test]
async fn auth_failure_maps_to_api_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/sessions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let client = client(&server.uri(), Auth::None);
    let err = client.sessions_list().await.unwrap_err();
    let lab_apis::openacp::OpenAcpError::Api(api) = err;
    assert_eq!(api.kind(), "auth_failed");
}

#[tokio::test]
async fn session_prompt_encodes_session_id_and_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/sessions/sess%2Fone/prompt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true
        })))
        .mount(&server)
        .await;

    let client = client(&server.uri(), Auth::None);
    let result = client
        .session_prompt(
            "sess/one",
            &PromptRequest {
                prompt: "do the thing".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(result["ok"], true);
}
