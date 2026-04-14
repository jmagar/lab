#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Unit tests for new `RadarrClient` SDK methods — success and error paths
//! using a wiremock server. No real Radarr instance required.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path, path_regex},
};

use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;

fn client(base_url: &str) -> RadarrClient {
    RadarrClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "testkey".into(),
        },
    )
    .expect("RadarrClient::new")
}

// ── system.logs ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn system_logs_accepts_contents_url_field() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/log/file"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 50,
                "filename": "radarr.txt",
                "lastWriteTime": "2026-04-14T13:40:12Z",
                "contentsUrl": "/api/v1//radarr.txt",
                "downloadUrl": "/logfile/radarr.txt"
            }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri()).log_files().await.expect("log_files");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].filename, "radarr.txt");
}

// ── movie.edit ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn movie_edit_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v3/movie/42"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42,
            "title": "Updated Movie",
            "year": 2023,
            "tmdbId": 12345,
            "monitored": true,
            "hasFile": false,
            "sizeOnDisk": 0
        })))
        .mount(&server)
        .await;

    let body = serde_json::json!({ "id": 42, "title": "Updated Movie" });
    let result = client(&server.uri())
        .movie_edit(lab_apis::radarr::types::movie::MovieId(42), &body)
        .await
        .expect("movie_edit");

    assert_eq!(result["id"], 42);
    assert_eq!(result["title"], "Updated Movie");
}

#[tokio::test]
async fn movie_edit_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v3/movie/999"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let body = serde_json::json!({ "id": 999 });
    let err = client(&server.uri())
        .movie_edit(lab_apis::radarr::types::movie::MovieId(999), &body)
        .await
        .expect_err("should fail with not found");

    assert!(matches!(
        err,
        lab_apis::radarr::error::RadarrError::NotFound {
            kind: "movie",
            id: 999
        }
    ));
}

// ── wanted.missing ───────────────────────────────────────────────────────────

#[tokio::test]
async fn wanted_missing_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/wanted/missing"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "pageSize": 10,
            "totalRecords": 2,
            "records": [
                { "id": 1, "title": "Missing Movie 1" },
                { "id": 2, "title": "Missing Movie 2" }
            ]
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .wanted_missing(1, 10)
        .await
        .expect("wanted_missing");

    assert_eq!(result["totalRecords"], 2);
    assert_eq!(result["records"].as_array().unwrap().len(), 2);
}

// ── wanted.cutoff ────────────────────────────────────────────────────────────

#[tokio::test]
async fn wanted_cutoff_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/wanted/cutoff"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "pageSize": 10,
            "totalRecords": 1,
            "records": [
                { "id": 5, "title": "Below Cutoff Movie" }
            ]
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .wanted_cutoff(1, 10)
        .await
        .expect("wanted_cutoff");

    assert_eq!(result["totalRecords"], 1);
}

// ── release.grab ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn release_grab_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/release"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "guid": "abc123",
            "title": "Some Movie 2023",
            "indexer": "MyIndexer"
        })))
        .mount(&server)
        .await;

    let release = serde_json::json!({ "guid": "abc123", "indexerId": 1 });
    let result = client(&server.uri())
        .release_grab(&release)
        .await
        .expect("release_grab");

    assert_eq!(result["guid"], "abc123");
}

// ── history.movie ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn history_movie_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/history/movie"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 10,
                "movieId": 42,
                "sourceTitle": "Some.Movie.2023",
                "eventType": "grabbed",
                "date": "2024-01-01T00:00:00Z",
                "data": {}
            }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .history_movie(lab_apis::radarr::types::movie::MovieId(42))
        .await
        .expect("history_movie");

    assert!(result.is_array() || result.is_object());
}

// ── history.failed-retry ─────────────────────────────────────────────────────

#[tokio::test]
async fn history_failed_retry_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex("/api/v3/history/failed/[0-9]+"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .history_failed_retry(lab_apis::radarr::types::history::HistoryId(7))
        .await
        .expect("history_failed_retry");
}

// ── blocklist.delete ──────────────────────────────────────────────────────────

#[tokio::test]
async fn blocklist_delete_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v3/blocklist/3"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server.uri())
        .blocklist_delete(lab_apis::radarr::types::history::BlocklistId(3))
        .await
        .expect("blocklist_delete");
}

// ── customformat.list ─────────────────────────────────────────────────────────

#[tokio::test]
async fn customformat_list_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/customformat"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "name": "HDR",
                "includeCustomFormatWhenRenaming": false,
                "specifications": []
            }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .customformat_list()
        .await
        .expect("customformat_list");

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(result[0]["name"], "HDR");
}

// ── system.restart ────────────────────────────────────────────────────────────

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

// ── system.backup ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn system_backup_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/system/backup"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "name": "radarr_backup_2024.zip",
                "type": "scheduled",
                "path": "/config/Backups/scheduled/radarr_backup_2024.zip",
                "time": "2024-01-01T00:00:00Z"
            }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .system_backup()
        .await
        .expect("system_backup");

    assert!(result.is_array());
    assert_eq!(result[0]["name"], "radarr_backup_2024.zip");
}

// ── system.task ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn system_tasks_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v3/system/task"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1,
                "name": "RssSync",
                "taskName": "RssSync",
                "interval": 60,
                "lastExecution": "2024-01-01T00:00:00Z",
                "nextExecution": "2024-01-01T01:00:00Z",
                "lastDuration": "00:00:05"
            }
        ])))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .system_tasks()
        .await
        .expect("system_tasks");

    assert!(result.is_array());
    assert_eq!(result[0]["name"], "RssSync");
}

// ── system.task-execute ───────────────────────────────────────────────────────

#[tokio::test]
async fn system_task_execute_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/command"))
        .and(header("X-Api-Key", "testkey"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 99,
            "name": "RssSync",
            "commandName": "RssSync",
            "status": "queued",
            "queued": "2024-01-01T00:00:00Z",
            "started": null,
            "ended": null,
            "duration": null,
            "trigger": "manual",
            "stateChangeTime": "2024-01-01T00:00:00Z",
            "sendUpdatesToClient": true,
            "updateScheduledTask": true,
            "lastExecutionTime": "2024-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .system_task_execute("RssSync")
        .await
        .expect("system_task_execute");

    assert_eq!(result.id.0, 99);
    assert_eq!(result.name, "RssSync");
}
