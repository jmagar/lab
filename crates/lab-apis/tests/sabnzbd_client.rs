#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Wiremock-based tests for new `SabnzbdClient` methods.
//! No real SABnzbd instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use lab_apis::sabnzbd::SabnzbdClient;

const API_KEY: &str = "testapikey";

fn client(base_url: &str) -> SabnzbdClient {
    SabnzbdClient::new(base_url, API_KEY.to_string()).expect("SabnzbdClient::new")
}

// ── queue.addurl ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn queue_addurl_sends_mode_addurl() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "addurl"))
        .and(query_param("apikey", API_KEY))
        .and(query_param("name", "https://example.com/file.nzb"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true,
            "nzo_ids": ["SABnzb+abc123"]
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .queue_addurl("https://example.com/file.nzb", None, None)
        .await
        .expect("queue_addurl");
    assert_eq!(resp["status"], true);
}

#[tokio::test]
async fn queue_addurl_with_cat_and_priority() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "addurl"))
        .and(query_param("cat", "movies"))
        .and(query_param("priority", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true,
            "nzo_ids": ["SABnzb+def456"]
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .queue_addurl("https://example.com/movie.nzb", Some("movies"), Some("1"))
        .await
        .expect("queue_addurl with options");
    assert_eq!(resp["status"], true);
}

// ── history.retry ────────────────────────────────────────────────────────────

#[tokio::test]
async fn history_retry_sends_mode_retry_with_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "retry"))
        .and(query_param("value", "SABnzbd_nzo_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let ok = client(&server.uri())
        .history_retry("SABnzbd_nzo_abc123")
        .await
        .expect("history_retry");
    assert!(ok);
}

// ── history.retry_all ────────────────────────────────────────────────────────

#[tokio::test]
async fn history_retry_all_sends_mode_retry_all() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "retry_all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let ok = client(&server.uri())
        .history_retry_all()
        .await
        .expect("history_retry_all");
    assert!(ok);
}

// ── server.fullstatus ────────────────────────────────────────────────────────

#[tokio::test]
async fn server_fullstatus_sends_mode_fullstatus() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "fullstatus"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": {
                "paused": false,
                "version": "3.7.1",
                "uptime": 12345
            }
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .server_fullstatus()
        .await
        .expect("server_fullstatus");
    assert!(resp["status"]["paused"] == false);
}

// ── category.list ────────────────────────────────────────────────────────────

#[tokio::test]
async fn category_list_sends_mode_get_cats() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "get_cats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "categories": ["Default", "Movies", "TV", "Audio"]
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .category_list()
        .await
        .expect("category_list");
    assert!(resp["categories"].is_array());
}

// ── queue.set_complete_action ────────────────────────────────────────────────

#[tokio::test]
async fn queue_set_complete_action_sends_correct_mode_and_value() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "change_complete_action"))
        .and(query_param("value", "shutdown_pc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let ok = client(&server.uri())
        .queue_set_complete_action("shutdown_pc")
        .await
        .expect("queue_set_complete_action");
    assert!(ok);
}

// ── pp.pause ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn pp_pause_sends_mode_pause_pp() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "pause_pp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let ok = client(&server.uri()).pp_pause().await.expect("pp_pause");
    assert!(ok);
}

// ── pp.resume ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn pp_resume_sends_mode_resume_pp() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "resume_pp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let ok = client(&server.uri()).pp_resume().await.expect("pp_resume");
    assert!(ok);
}

// ── rss.fetch_now ────────────────────────────────────────────────────────────

#[tokio::test]
async fn rss_fetch_now_sends_mode_rss_now() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "rss_now"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": true
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .rss_fetch_now()
        .await
        .expect("rss_fetch_now");
    assert_eq!(resp["status"], true);
}

// ── config.get ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn config_get_sends_mode_get_config() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api"))
        .and(query_param("mode", "get_config"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "config": {
                "misc": { "host": "0.0.0.0" },
                "servers": [],
                "categories": []
            }
        })))
        .mount(&server)
        .await;

    let resp = client(&server.uri())
        .config_get()
        .await
        .expect("config_get");
    assert!(resp["config"].is_object());
}
