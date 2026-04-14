#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::apprise::{
    AppriseClient,
    types::{BodyFormat, NotifyRequest, NotifyType},
};
use lab_apis::core::Auth;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, body_partial_json, method, path},
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

#[tokio::test]
async fn add_config_posts_json_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/add/mykey"))
        .and(body_partial_json(serde_json::json!({
            "config": "urls:\n- slack://...",
            "format": "yaml"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    client
        .add_config("mykey", "urls:\n- slack://...", "yaml")
        .await
        .expect("add_config");
}

#[tokio::test]
async fn get_config_returns_yaml_text() {
    let server = MockServer::start().await;
    let yaml = "urls:\n- slack://t/a/b\n";

    Mock::given(method("POST"))
        .and(path("/get/mykey"))
        .respond_with(ResponseTemplate::new(200).set_body_string(yaml))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    let result = client.get_config("mykey").await.expect("get_config");
    assert_eq!(result, yaml);
}

#[tokio::test]
async fn delete_config_posts_to_del_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/del/mykey"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    client.delete_config("mykey").await.expect("delete_config");
}

#[tokio::test]
async fn get_urls_returns_url_list() {
    let server = MockServer::start().await;
    let body = serde_json::json!([
        {"url": "slack://t/a/b", "tags": ["devops"]},
        {"url": "mailto://user@example.com", "tags": []}
    ]);

    Mock::given(method("GET"))
        .and(path("/json/urls/mykey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    let urls = client.get_urls("mykey").await.expect("get_urls");
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0].url, "slack://t/a/b");
    assert_eq!(urls[0].tags, vec!["devops"]);
    assert!(urls[1].tags.is_empty());
}

#[tokio::test]
async fn server_details_returns_json_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "version": "1.1",
        "plugins": {"count": 100}
    });

    Mock::given(method("GET"))
        .and(path("/details"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = AppriseClient::new(&server.uri(), Auth::None).expect("client");
    let details = client.details().await.expect("server_details");
    assert_eq!(details["version"], "1.1");
    assert_eq!(details["plugins"]["count"], 100);
}
