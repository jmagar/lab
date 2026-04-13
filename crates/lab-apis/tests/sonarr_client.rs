#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for `SonarrClient` — success and error paths
//! using a wiremock server. No real Sonarr instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

use lab_apis::core::Auth;
use lab_apis::sonarr::SonarrClient;

fn client(base_url: &str) -> SonarrClient {
    SonarrClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "testkey".into(),
        },
    )
    .expect("SonarrClient::new")
}

// ── Success paths ────────────────────────────────────────────────────────────

#[tokio::test]
async fn series_list_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/series"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "title": "Breaking Bad",
                "status": "ended",
                "overview": "A chemistry teacher becomes a drug lord.",
                "path": "/tv/breaking-bad",
                "monitored": true,
                "seasonCount": 5
            }
        ])))
        .mount(&server)
        .await;

    let series_list = client(&server.uri()).series_list().await.expect("series_list");
    assert_eq!(series_list.len(), 1);
    assert_eq!(series_list[0].title, "Breaking Bad");
    assert_eq!(series_list[0].id, 1);
    assert!(series_list[0].monitored);
    assert_eq!(series_list[0].season_count, 5);
}

#[tokio::test]
async fn system_status_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/system/status"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "4.0.0.123",
            "appName": "Sonarr",
            "branch": "main",
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
            "runtimeVersion": "6.0.0",
            "runtimeName": ".NET"
        })))
        .mount(&server)
        .await;

    let status = client(&server.uri()).system_status().await.expect("system_status");
    // Verify the response decoded as Value successfully
    assert!(status.get("version").is_some());
    assert_eq!(status["version"], "4.0.0.123");
    assert_eq!(status["appName"], "Sonarr");
}

// ── Error / transport failure paths ──────────────────────────────────────────

#[tokio::test]
async fn series_list_auth_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/series"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let result = client(&server.uri()).series_list().await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("auth") || err.to_string().contains("401"),
        "expected auth error, got: {err}"
    );
}

#[tokio::test]
async fn health_check_failure() {
    // Server is started but no routes mounted — will refuse connection or return 404.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/health"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = client(&server.uri()).health().await;
    assert!(result.is_err(), "expected error on 500 response");
}
