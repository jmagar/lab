#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Wiremock-based tests for the 12 new `PlexClient` methods.
//!
//! Plex auth: `X-Plex-Token` header; `Accept: application/json`.
//! Tests use a wiremock server — no real Plex instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use lab_apis::core::Auth;
use lab_apis::plex::PlexClient;

const TOKEN: &str = "test-plex-token";

fn client(base_url: &str) -> PlexClient {
    PlexClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-Plex-Token".into(),
            key: TOKEN.to_string(),
        },
    )
    .expect("PlexClient::new")
}

fn json_ok() -> serde_json::Value {
    serde_json::json!({ "MediaContainer": { "size": 0 } })
}

// ── library.browse ────────────────────────────────────────────────────────────

#[tokio::test]
async fn library_browse_no_filters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/sections/1/all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .library_browse(1, None, None)
        .await
        .expect("library_browse");
    assert!(result.get("MediaContainer").is_some());
}

#[tokio::test]
async fn library_browse_with_type_and_sort() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/sections/2/all"))
        .and(query_param("type", "1"))
        .and(query_param("sort", "titleSort"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .library_browse(2, Some("1"), Some("titleSort"))
        .await
        .expect("library_browse with filters");
    assert!(result.get("MediaContainer").is_some());
}

// ── library.empty-trash ───────────────────────────────────────────────────────

#[tokio::test]
async fn library_empty_trash() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/library/sections/3/emptyTrash"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .library_empty_trash(3)
        .await
        .expect("library_empty_trash");
}

// ── metadata.delete ───────────────────────────────────────────────────────────

#[tokio::test]
async fn metadata_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/library/metadata/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .metadata_delete("12345")
        .await
        .expect("metadata_delete");
}

// ── metadata.edit ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn metadata_edit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/library/metadata/99"))
        .and(query_param("title", "New Title"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let fields = serde_json::json!({ "title": "New Title" });
    let result = client(&server.uri())
        .metadata_edit("99", &fields)
        .await
        .expect("metadata_edit");
    assert!(result.get("MediaContainer").is_some());
}

// ── metadata.refresh ──────────────────────────────────────────────────────────

#[tokio::test]
async fn metadata_refresh() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/library/metadata/42/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .metadata_refresh("42")
        .await
        .expect("metadata_refresh");
}

// ── session.history ───────────────────────────────────────────────────────────

#[tokio::test]
async fn session_history_no_filters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/status/sessions/history/all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .session_history(None, None)
        .await
        .expect("session_history");
    assert!(result.get("MediaContainer").is_some());
}

#[tokio::test]
async fn session_history_with_account_and_limit() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/status/sessions/history/all"))
        .and(query_param("accountID", "7"))
        .and(query_param("X-Plex-Container-Size", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .session_history(Some(7), Some(50))
        .await
        .expect("session_history with filters");
    assert!(result.get("MediaContainer").is_some());
}

// ── hubs.continue-watching ────────────────────────────────────────────────────

#[tokio::test]
async fn hubs_continue_watching() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/hubs/continueWatching"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json_ok()))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .hubs_continue_watching()
        .await
        .expect("hubs_continue_watching");
    assert!(result.get("MediaContainer").is_some());
}

// ── butler.list ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn butler_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/butler"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ButlerTasks": []
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .butler_list()
        .await
        .expect("butler_list");
    assert!(result.get("ButlerTasks").is_some());
}

// ── butler.run ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn butler_run() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/butler/BackupDatabase"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .butler_run("BackupDatabase")
        .await
        .expect("butler_run");
}

// ── item.scrobble ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn item_scrobble() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/:/scrobble"))
        .and(query_param("key", "555"))
        .and(query_param("identifier", "com.plexapp.plugins.library"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .item_scrobble("555")
        .await
        .expect("item_scrobble");
}

// ── item.unscrobble ───────────────────────────────────────────────────────────

#[tokio::test]
async fn item_unscrobble() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/:/unscrobble"))
        .and(query_param("key", "555"))
        .and(query_param("identifier", "com.plexapp.plugins.library"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&server)
        .await;

    client(&server.uri())
        .item_unscrobble("555")
        .await
        .expect("item_unscrobble");
}

// ── updater.status ────────────────────────────────────────────────────────────

#[tokio::test]
async fn updater_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/updater/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "MediaContainer": { "canInstall": false, "canCheck": true }
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .updater_status()
        .await
        .expect("updater_status");
    assert!(result.get("MediaContainer").is_some());
}
