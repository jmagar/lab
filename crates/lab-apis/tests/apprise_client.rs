#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::apprise::{
    AppriseClient,
    types::{BodyFormat, NotifyRequest, NotifyType},
};
use lab_apis::core::Auth;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

#[tokio::test]
async fn health_uses_status_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/status"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    client.health().await.expect("health");
}

#[tokio::test]
async fn notify_posts_json_body() {
    let server = MockServer::start().await;
    let request = NotifyRequest {
        body: "test message".into(),
        title: Some("Alert".into()),
        urls: Some(vec!["mailto://user:pass@example.com".into()]),
        notify_type: Some(NotifyType::Warning),
        format: Some(BodyFormat::Markdown),
        tag: None,
    };

    Mock::given(method("POST"))
        .and(path("/notify"))
        .and(body_json(&request))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    client.notify(&request).await.expect("notify");
}

#[tokio::test]
async fn notify_key_posts_json_body() {
    let server = MockServer::start().await;
    let request = NotifyRequest {
        body: "test message".into(),
        title: None,
        urls: None,
        notify_type: Some(NotifyType::Info),
        format: Some(BodyFormat::Text),
        tag: Some("devops".into()),
    };

    Mock::given(method("POST"))
        .and(path("/notify/ops"))
        .and(body_json(&request))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    client
        .notify_key("ops", &request)
        .await
        .expect("notify_key");
}
