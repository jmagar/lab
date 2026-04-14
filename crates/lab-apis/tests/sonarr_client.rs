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

    let series_list = client(&server.uri())
        .series_list()
        .await
        .expect("series_list");
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

    let status = client(&server.uri())
        .system_status()
        .await
        .expect("system_status");
    // Verify the response decoded as Value successfully
    assert!(status.get("version").is_some());
    assert_eq!(status["version"], "4.0.0.123");
    assert_eq!(status["appName"], "Sonarr");
}

// ── New action success paths ──────────────────────────────────────────────────

#[tokio::test]
async fn series_edit_success() {
    let server = MockServer::start().await;
    let series_body = serde_json::json!({
        "id": 1,
        "title": "Breaking Bad",
        "status": "ended",
        "overview": "A chemistry teacher becomes a drug lord.",
        "path": "/tv/breaking-bad",
        "monitored": false,
        "seasonCount": 5
    });
    Mock::given(method("PUT"))
        .and(path("/api/v3/series/1"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(202).set_body_json(series_body.clone()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .series_edit(1, &series_body)
        .await
        .expect("series_edit");
    assert_eq!(result["id"], 1);
}

#[tokio::test]
async fn episode_monitor_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v3/episode/monitor"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .episode_monitor(&[1, 2, 3], true)
        .await
        .expect("episode_monitor");
    assert!(result.is_object());
}

#[tokio::test]
async fn wanted_cutoff_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/wanted/cutoff"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "pageSize": 10,
            "totalRecords": 0,
            "records": []
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .wanted_cutoff(Some(1), Some(10))
        .await
        .expect("wanted_cutoff");
    assert_eq!(result["totalRecords"], 0);
}

#[tokio::test]
async fn release_search_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/release"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .release_search(Some(1), Some(1))
        .await
        .expect("release_search");
    assert!(result.is_array());
}

#[tokio::test]
async fn release_grab_success() {
    let server = MockServer::start().await;
    let body = serde_json::json!({ "guid": "abc123" });
    Mock::given(method("POST"))
        .and(path("/api/v3/release"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "guid": "abc123" })),
        )
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .release_grab(&body)
        .await
        .expect("release_grab");
    assert_eq!(result["guid"], "abc123");
}

#[tokio::test]
async fn history_series_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/history/series"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .history_series(1)
        .await
        .expect("history_series");
    assert!(result.is_array());
}

#[tokio::test]
async fn history_failed_retry_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/history/failed/42"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    client(&server.uri())
        .history_failed_retry(42)
        .await
        .expect("history_failed_retry");
}

#[tokio::test]
async fn blocklist_list_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/blocklist"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "pageSize": 10,
            "totalRecords": 0,
            "records": []
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .blocklist_list()
        .await
        .expect("blocklist_list");
    assert_eq!(result["totalRecords"], 0);
}

#[tokio::test]
async fn blocklist_delete_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v3/blocklist/5"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .blocklist_delete(5)
        .await
        .expect("blocklist_delete");
}

#[tokio::test]
async fn episodefile_delete_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v3/episodefile/99"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .episodefile_delete(99)
        .await
        .expect("episodefile_delete");
}

#[tokio::test]
async fn system_restart_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/system/restart"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .system_restart()
        .await
        .expect("system_restart");
}

#[tokio::test]
async fn system_backup_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/system/backup"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "id": 1, "name": "sonarr_backup_2024.zip", "type": "manual" }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .system_backup()
        .await
        .expect("system_backup");
    assert!(result.is_array());
    assert_eq!(result[0]["name"], "sonarr_backup_2024.zip");
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
