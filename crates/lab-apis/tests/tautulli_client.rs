#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Wiremock-based tests for the 11 new `TautulliClient` methods.
//!
//! All Tautulli API calls are GET /api/v2 with `apikey` + `cmd` query params.
//! Tests use a wiremock server — no real Tautulli instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use lab_apis::tautulli::TautulliClient;

const API_KEY: &str = "testkey";

fn client(base_url: &str) -> TautulliClient {
    TautulliClient::new(base_url, API_KEY.to_string()).expect("TautulliClient::new")
}

/// Helper: build a standard Tautulli API success envelope.
fn tautulli_ok(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "response": {
            "result": "success",
            "data": data
        }
    })
}

// ── media.recently-added ─────────────────────────────────────────────────────

#[tokio::test]
async fn recently_added_default_count() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("apikey", API_KEY))
        .and(query_param("cmd", "get_recently_added"))
        .and(query_param("count", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "recently_added": []
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_recently_added(None, None)
        .await
        .expect("get_recently_added");
    assert!(result.get("recently_added").is_some());
}

#[tokio::test]
async fn recently_added_with_count_and_section() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_recently_added"))
        .and(query_param("count", "10"))
        .and(query_param("section_id", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "recently_added": [{"title": "Dune"}]
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_recently_added(Some(10), Some("1"))
        .await
        .expect("get_recently_added with section");
    let items = result["recently_added"].as_array().unwrap();
    assert_eq!(items.len(), 1);
}

// ── media.metadata ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_metadata_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_metadata"))
        .and(query_param("rating_key", "12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "title": "The Matrix",
            "rating_key": "12345",
            "media_type": "movie"
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_metadata("12345")
        .await
        .expect("get_metadata");
    assert_eq!(result["title"], "The Matrix");
    assert_eq!(result["media_type"], "movie");
}

// ── media.children ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_children_metadata_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_children_metadata"))
        .and(query_param("rating_key", "999"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "children_list": [{"title": "Episode 1"}]
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_children_metadata("999")
        .await
        .expect("get_children_metadata");
    assert!(result["children_list"].is_array());
}

// ── user.item-stats ───────────────────────────────────────────────────────────

#[tokio::test]
async fn get_item_user_stats_no_media_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_item_user_stats"))
        .and(query_param("rating_key", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!([
            {"user_id": 1, "plays": 5}
        ]))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_item_user_stats("42", None)
        .await
        .expect("get_item_user_stats");
    assert!(result.as_array().is_some());
}

#[tokio::test]
async fn get_item_user_stats_with_media_type() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_item_user_stats"))
        .and(query_param("rating_key", "42"))
        .and(query_param("media_type", "movie"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!([
            {"user_id": 2, "plays": 3}
        ]))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_item_user_stats("42", Some("movie"))
        .await
        .expect("get_item_user_stats with media_type");
    let arr = result.as_array().unwrap();
    assert_eq!(arr[0]["plays"], 3);
}

// ── plays.by-day ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_plays_by_dayofweek_default() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_plays_by_dayofweek"))
        .and(query_param("time_range", "30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "categories": ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"],
            "series": []
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_plays_by_dayofweek(None)
        .await
        .expect("get_plays_by_dayofweek");
    assert!(result["categories"].is_array());
}

#[tokio::test]
async fn get_plays_by_dayofweek_custom_range() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_plays_by_dayofweek"))
        .and(query_param("time_range", "7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "series": [{"name": "Movies", "data": [1,2,3,4,5,6,7]}]
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_plays_by_dayofweek(Some(7))
        .await
        .expect("get_plays_by_dayofweek custom");
    assert!(result["series"].is_array());
}

// ── plays.by-hour ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_plays_by_hourofday_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_plays_by_hourofday"))
        .and(query_param("time_range", "30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "categories": [],
            "series": []
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_plays_by_hourofday(None)
        .await
        .expect("get_plays_by_hourofday");
    assert!(result.is_object());
}

// ── plays.by-stream-type ──────────────────────────────────────────────────────

#[tokio::test]
async fn get_plays_by_stream_type_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_plays_by_stream_type"))
        .and(query_param("time_range", "30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "categories": ["Direct Play", "Transcode"],
            "series": []
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_plays_by_stream_type(None)
        .await
        .expect("get_plays_by_stream_type");
    assert!(result["categories"].is_array());
}

// ── plays.by-month ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_plays_per_month_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_plays_per_month"))
        .and(query_param("time_range", "12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "categories": ["Jan","Feb","Mar"],
            "series": []
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_plays_per_month(Some(12))
        .await
        .expect("get_plays_per_month");
    let cats = result["categories"].as_array().unwrap();
    assert_eq!(cats.len(), 3);
}

// ── server.pms-update ─────────────────────────────────────────────────────────

#[tokio::test]
async fn get_pms_update_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_pms_update"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "update_available": false,
            "version": "1.32.5"
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .get_pms_update()
        .await
        .expect("get_pms_update");
    assert_eq!(result["update_available"], false);
}

// ── media.export-metadata ────────────────────────────────────────────────────

#[tokio::test]
async fn export_metadata_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "export_metadata"))
        .and(query_param("rating_key", "77"))
        .and(query_param("media_type", "movie"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "export_id": 1,
            "file_format": "csv"
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .export_metadata("77", "movie")
        .await
        .expect("export_metadata");
    assert_eq!(result["file_format"], "csv");
}

// ── user.delete-history ───────────────────────────────────────────────────────

#[tokio::test]
async fn delete_all_user_history_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "delete_all_user_history"))
        .and(query_param("user_id", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tautulli_ok(serde_json::json!({
            "deleted": true
        }))))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .delete_all_user_history(42)
        .await
        .expect("delete_all_user_history");
    assert_eq!(result["deleted"], true);
}

// ── Error path: 404 propagates ────────────────────────────────────────────────

#[tokio::test]
async fn get_metadata_404_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2"))
        .and(query_param("cmd", "get_metadata"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let result = client(&server.uri()).get_metadata("0").await;
    assert!(result.is_err());
}
