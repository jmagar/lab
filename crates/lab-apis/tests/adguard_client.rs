use lab_apis::adguard::AdguardClient;
use lab_apis::core::Auth;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> AdguardClient {
    AdguardClient::new(
        &server.uri(),
        Auth::Session {
            cookie: "agh_session=secret".to_string(),
        },
    )
    .unwrap()
}

#[tokio::test]
async fn version_uses_control_version() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/control/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v0.107.52"))
        .mount(&server)
        .await;

    let response = client(&server).version().await.unwrap();

    assert_eq!(response.value["version"], "v0.107.52");
}

#[tokio::test]
async fn querylog_sends_session_cookie_and_redacts_client_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/control/querylog"))
        .and(header("cookie", "agh_session=secret"))
        .and(query_param("limit", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "domain": "example.com",
                "client": "192.0.2.10",
                "client_ip": "192.0.2.10",
                "upstream": "1.1.1.1"
            }]
        })))
        .mount(&server)
        .await;

    let response = client(&server).querylog(1, None, None).await.unwrap();
    let item = &response.value["data"][0];

    assert_eq!(item["domain"], "example.com");
    assert!(item.get("client").is_none());
    assert!(item.get("client_ip").is_none());
    assert!(item.get("upstream").is_none());
}

#[tokio::test]
async fn querylog_rejects_out_of_range_limit() {
    let server = MockServer::start().await;
    let err = client(&server).querylog(0, None, None).await.unwrap_err();

    assert!(err.to_string().contains("limit must be between 1 and 200"));
}
