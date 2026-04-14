#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Wiremock-based tests for new `QbittorrentClient` methods.
//! No real qBittorrent instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use lab_apis::qbittorrent::QbittorrentClient;

fn client(base_url: &str) -> QbittorrentClient {
    QbittorrentClient::new(base_url, "SID=testsession".to_string()).expect("QbittorrentClient::new")
}

// ── torrent.add ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn add_torrent_sends_post_to_add() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Ok."))
        .mount(&server)
        .await;

    client(&server.uri())
        .add_torrent("magnet:?xt=urn:btih:abc123", None, None, None)
        .await
        .expect("add_torrent");
}

#[tokio::test]
async fn add_torrent_with_category_and_tags() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/add"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Ok."))
        .mount(&server)
        .await;

    client(&server.uri())
        .add_torrent(
            "magnet:?xt=urn:btih:abc123",
            Some("/downloads/movies"),
            Some("movies"),
            Some("hd,bluray"),
        )
        .await
        .expect("add_torrent with options");
}

// ── transfer.toggle-speed-limits ─────────────────────────────────────────────

#[tokio::test]
async fn toggle_speed_limits_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/transfer/toggleSpeedLimitsMode"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .toggle_speed_limits()
        .await
        .expect("toggle_speed_limits");
}

// ── torrent.files ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn torrent_files_returns_file_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/files"))
        .and(query_param("hash", "abc123hash"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "index": 0,
                "name": "movie.mkv",
                "size": 1_073_741_824i64,
                "progress": 1.0,
                "priority": 1,
                "is_seed": true,
                "piece_range": [0, 1023],
                "availability": 1.0
            }
        ])))
        .mount(&server)
        .await;

    let files = client(&server.uri())
        .torrent_files("abc123hash")
        .await
        .expect("torrent_files");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "movie.mkv");
    assert_eq!(files[0].priority, 1);
}

// ── torrent.set-file-prio ────────────────────────────────────────────────────

#[tokio::test]
async fn set_file_priority_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/filePrio"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .set_file_priority("abc123hash", "0|1|2", 6)
        .await
        .expect("set_file_priority");
}

// ── torrent.set-location ─────────────────────────────────────────────────────

#[tokio::test]
async fn set_location_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/setLocation"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .set_location("abc123hash", "/new/path")
        .await
        .expect("set_location");
}

// ── torrent.add-tags ─────────────────────────────────────────────────────────

#[tokio::test]
async fn add_tags_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/addTags"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .add_tags("abc123hash", "hd,bluray")
        .await
        .expect("add_tags");
}

// ── torrent.remove-tags ──────────────────────────────────────────────────────

#[tokio::test]
async fn remove_tags_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/removeTags"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .remove_tags("abc123hash", "hd")
        .await
        .expect("remove_tags");
}

// ── torrent.reannounce ───────────────────────────────────────────────────────

#[tokio::test]
async fn reannounce_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/reannounce"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .reannounce("all")
        .await
        .expect("reannounce");
}

// ── torrent.set-share-limits ─────────────────────────────────────────────────

#[tokio::test]
async fn set_share_limits_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/setShareLimits"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .set_share_limits("abc123hash", 2.0, 1440, -1)
        .await
        .expect("set_share_limits");
}

// ── category.create ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_category_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/createCategory"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .create_category("movies", Some("/downloads/movies"))
        .await
        .expect("create_category");
}

#[tokio::test]
async fn create_category_without_savepath() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/createCategory"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .create_category("tv", None)
        .await
        .expect("create_category without savepath");
}

// ── category.edit ────────────────────────────────────────────────────────────

#[tokio::test]
async fn edit_category_sends_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/editCategory"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .edit_category("movies", "/new/downloads/movies")
        .await
        .expect("edit_category");
}

// ── sync.maindata ────────────────────────────────────────────────────────────

#[tokio::test]
async fn sync_maindata_full_dump_at_rid_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/sync/maindata"))
        .and(query_param("rid", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rid": 5,
            "full_update": true,
            "torrents": {},
            "categories": {},
            "tags": [],
            "server_state": {
                "connection_status": "connected",
                "dl_info_speed": 0,
                "up_info_speed": 0
            }
        })))
        .mount(&server)
        .await;

    let data = client(&server.uri())
        .sync_maindata(0)
        .await
        .expect("sync_maindata rid=0");
    assert_eq!(data["rid"], 5);
    assert_eq!(data["full_update"], true);
}

#[tokio::test]
async fn sync_maindata_incremental_delta() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/sync/maindata"))
        .and(query_param("rid", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rid": 6,
            "full_update": false,
            "torrents_removed": [],
            "categories_removed": [],
            "tags_removed": []
        })))
        .mount(&server)
        .await;

    let data = client(&server.uri())
        .sync_maindata(5)
        .await
        .expect("sync_maindata rid=5");
    assert_eq!(data["rid"], 6);
    assert_eq!(data["full_update"], false);
}
