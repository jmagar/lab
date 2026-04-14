#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Wiremock-based tests for the new ProwlarrClient methods.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path, query_param},
};

use lab_apis::core::Auth;
use lab_apis::prowlarr::ProwlarrClient;

fn make_client(uri: &str) -> ProwlarrClient {
    ProwlarrClient::new(
        uri,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "testkey".into(),
        },
    )
    .expect("ProwlarrClient::new")
}

// ── indexer.edit ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_edit_ok() {
    let server = MockServer::start().await;
    let body = serde_json::json!({ "name": "Updated", "enable": true });
    let resp = serde_json::json!({ "id": 1, "name": "Updated", "enable": true });

    Mock::given(method("PUT"))
        .and(path("/api/v1/indexer/1"))
        .and(header("X-Api-Key", "testkey"))
        .and(body_json(body.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp.clone()))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.indexer_edit(1, &body).await.expect("indexer_edit");
    assert_eq!(result["name"], "Updated");
}

// ── indexer.add ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_add_ok() {
    let server = MockServer::start().await;
    let body = serde_json::json!({ "name": "NewIndexer", "enable": true });
    let resp = serde_json::json!({ "id": 42, "name": "NewIndexer" });

    Mock::given(method("POST"))
        .and(path("/api/v1/indexer"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(201).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.indexer_add(&body).await.expect("indexer_add");
    assert_eq!(result["id"], 42);
}

// ── indexer.stats ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_stats_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!({ "indexers": [] });

    Mock::given(method("GET"))
        .and(path("/api/v1/indexerstats"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.indexer_stats().await.expect("indexer_stats");
}

// ── indexer.status ────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_status_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!([{ "indexerId": 1, "disabledTill": null }]);

    Mock::given(method("GET"))
        .and(path("/api/v1/indexerstatus"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.indexer_status().await.expect("indexer_status");
    assert!(result.is_array());
}

// ── indexer.search ────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_search_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!([{ "title": "test release", "guid": "abc123" }]);

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(header("X-Api-Key", "testkey"))
        .and(query_param("query", "ubuntu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .indexer_search("ubuntu", &[], &[], None)
        .await
        .expect("indexer_search");
    assert!(result.is_array());
    assert_eq!(result[0]["guid"], "abc123");
}

// ── indexer.grab ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn indexer_grab_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!({ "approved": true, "guid": "abc123" });

    Mock::given(method("POST"))
        .and(path("/api/v1/search"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.indexer_grab("abc123").await.expect("indexer_grab");
    assert_eq!(result["guid"], "abc123");
}

// ── history.indexer ───────────────────────────────────────────────────────────

#[tokio::test]
async fn history_indexer_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!({ "page": 1, "records": [] });

    Mock::given(method("GET"))
        .and(path("/api/v1/history"))
        .and(header("X-Api-Key", "testkey"))
        .and(query_param("indexerId", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.history_indexer(5).await.expect("history_indexer");
    assert_eq!(result["page"], 1);
}

// ── application.add ──────────────────────────────────────────────────────────

#[tokio::test]
async fn application_add_ok() {
    let server = MockServer::start().await;
    let body = serde_json::json!({ "name": "Radarr", "syncLevel": "fullSync" });
    let resp = serde_json::json!({ "id": 10, "name": "Radarr" });

    Mock::given(method("POST"))
        .and(path("/api/v1/applications"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(201).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .application_add(&body)
        .await
        .expect("application_add");
    assert_eq!(result["id"], 10);
}

// ── system.restart ────────────────────────────────────────────────────────────

#[tokio::test]
async fn system_restart_ok() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/system/restart"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.system_restart().await.expect("system_restart");
}

// ── system.backup ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn system_backup_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!([{ "id": 1, "name": "prowlarr_backup_2026.zip" }]);

    Mock::given(method("GET"))
        .and(path("/api/v1/system/backup"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.system_backup().await.expect("system_backup");
    assert!(result.is_array());
}

// ── tag.list ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tag_list_ok() {
    let server = MockServer::start().await;
    let resp = serde_json::json!([{ "id": 1, "label": "4k" }, { "id": 2, "label": "hd" }]);

    Mock::given(method("GET"))
        .and(path("/api/v1/tag"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(resp))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.tag_list().await.expect("tag_list");
    assert!(result.is_array());
    assert_eq!(result[0]["label"], "4k");
}
