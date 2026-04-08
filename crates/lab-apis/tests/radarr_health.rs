#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration test — `RadarrClient::system_status` must hit
//! `GET /api/v3/system/status` with the `X-Api-Key` header and decode the
//! minimal `SystemStatus` shape Radarr returns.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;

#[tokio::test]
async fn system_status_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/system/status"))
        .and(header("X-Api-Key", "abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "5.0.0.1234",
            "appName": "Radarr",
            "instanceName": "Radarr",
            "buildTime": "2024-01-01T00:00:00Z",
            "isDebug": false,
            "isProduction": true,
            "isAdmin": false,
            "isUserInteractive": false,
            "startupPath": "/app",
            "appData": "/config",
            "osName": "linux",
            "isLinux": true,
            "isOsx": false,
            "isWindows": false,
            "isDocker": true,
            "mode": "console",
            "branch": "main",
            "runtimeVersion": "6.0.0",
            "runtimeName": ".NET"
        })))
        .mount(&server)
        .await;

    let client = RadarrClient::new(
        &server.uri(),
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "abc123".into(),
        },
    );

    let status = client.system_status().await.expect("system_status");
    assert_eq!(status.version, "5.0.0.1234");
    assert_eq!(status.app_name, "Radarr");
    assert_eq!(status.instance_name.as_deref(), Some("Radarr"));
}
