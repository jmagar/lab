use lab_apis::core::Auth;
use lab_apis::glances::GlancesClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> GlancesClient {
    GlancesClient::new(
        &server.uri(),
        Auth::Bearer {
            token: "jwt".to_string(),
        },
    )
    .unwrap()
}

#[tokio::test]
async fn health_uses_status_endpoint_with_bearer_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/4/status"))
        .and(header("authorization", "Bearer jwt"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server).health().await.unwrap();
}

#[tokio::test]
async fn plugin_rejects_unknown_plugin_names() {
    let server = MockServer::start().await;
    let err = client(&server).plugin("../cpu").await.unwrap_err();

    assert!(err.to_string().contains("unsupported plugin"));
}

#[tokio::test]
async fn process_top_redacts_user_and_cmdline() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/4/processlist/top/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"pid": 42, "username": "root", "cmdline": "secret --token"}
        ])))
        .mount(&server)
        .await;

    let response = client(&server).process_top(1).await.unwrap();
    let item = &response.value[0];

    assert_eq!(item["pid"], 42);
    assert!(item.get("username").is_none());
    assert!(item.get("cmdline").is_none());
}
