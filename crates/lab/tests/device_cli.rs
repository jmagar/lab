use lab::config::{DevicePreferences, LabConfig};
use lab::device::master_client::MasterClient;
use url::Url;

#[tokio::test]
async fn device_list_command_reads_from_master_api() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/device/devices"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"device_id":"dookie","connected":true}
            ])),
        )
        .mount(&server)
        .await;

    let config = config_for_master(&server.uri());
    let value = lab::cli::device::fetch_devices(&config).await.unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn logs_search_command_reads_from_master_api() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/device/logs/search"))
        .and(wiremock::matchers::body_string_contains(
            "\"device_id\":\"dookie\"",
        ))
        .and(wiremock::matchers::body_string_contains(
            "\"query\":\"hello\"",
        ))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"device_id":"dookie","message":"hello"}
            ])),
        )
        .mount(&server)
        .await;

    let config = config_for_master(&server.uri());
    let value = lab::cli::logs::search_logs(&config, "dookie", "hello")
        .await
        .unwrap();
    assert_eq!(value.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn master_client_applies_bearer_token_to_master_requests() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/v1/device/devices"))
        .and(wiremock::matchers::header(
            "authorization",
            "Bearer shared-secret",
        ))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let value = MasterClient::with_bearer_token(server.uri(), Some("shared-secret".into()))
        .fetch_devices()
        .await
        .unwrap();
    assert!(value.as_array().unwrap().is_empty());
}

fn config_for_master(uri: &str) -> LabConfig {
    let parsed = Url::parse(uri).unwrap();
    let mut config = LabConfig::default();
    config.device = Some(DevicePreferences {
        master: parsed.host_str().map(str::to_string),
    });
    config.mcp.port = parsed.port();
    config
}
