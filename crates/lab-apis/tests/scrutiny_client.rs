use lab_apis::core::Auth;
use lab_apis::scrutiny::ScrutinyClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> ScrutinyClient {
    ScrutinyClient::new(
        &server.uri(),
        Auth::Bearer {
            token: "scrutiny-token".into(),
        },
    )
    .unwrap()
}

#[tokio::test]
async fn health_uses_health_endpoint_with_bearer_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .and(header("authorization", "Bearer scrutiny-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server).health().await.unwrap();
}

#[tokio::test]
async fn dashboard_summary_redacts_disk_identity_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "devices": [{
                "name": "disk",
                "serial": "secret-serial",
                "device_path": "/dev/sda",
                "smart_results": {"raw": true}
            }]
        })))
        .mount(&server)
        .await;

    let response = client(&server).dashboard_summary().await.unwrap();
    let device = &response.value["devices"][0];

    assert_eq!(device["name"], "disk");
    assert!(device.get("serial").is_none());
    assert!(device.get("device_path").is_none());
    assert!(device.get("smart_results").is_none());
}
