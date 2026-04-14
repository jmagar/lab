#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for `OverseerrClient` — wiremock-backed happy path and
//! error-path coverage.

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

use lab_apis::core::Auth;
use lab_apis::overseerr::OverseerrClient;

fn make_client(base_url: &str) -> OverseerrClient {
    OverseerrClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "test-api-key".into(),
        },
    )
    .expect("OverseerrClient::new should succeed")
}

#[tokio::test]
async fn status_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/status"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "version": "1.33.0",
            "commitTag": "abc123",
            "updateAvailable": false,
            "commitsBehind": 0,
            "restartRequired": false
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let status = client.status().await.expect("status should succeed");
    assert_eq!(status.version, "1.33.0");
    assert!(!status.update_available);
    assert!(!status.restart_required);
}

#[tokio::test]
async fn status_transport_error_returns_err() {
    // Use a port that nothing is listening on to force a network error.
    let client = make_client("http://127.0.0.1:1");
    let result = client.status().await;
    assert!(result.is_err(), "network error should produce Err");
}

#[tokio::test]
async fn request_list_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/request"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pageInfo": {
                "pages": 1,
                "pageSize": 20,
                "results": 1,
                "page": 1
            },
            "results": [
                {
                    "id": 42,
                    "status": 2,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let list = client
        .request_list(None, None, None, None, None)
        .await
        .expect("request_list should succeed");
    assert_eq!(list.results.len(), 1);
    assert_eq!(list.results[0].id, 42);
    assert_eq!(list.results[0].status, 2);
}

#[tokio::test]
async fn auth_failure_returns_auth_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/status"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.status().await;
    assert!(result.is_err());
    // The error should map to ApiError::Auth.
    let err_str = result.unwrap_err().to_string();
    // Error message will contain "auth" or "401".
    assert!(
        err_str.contains("auth") || err_str.contains("401") || err_str.contains("Unauthorized"),
        "Expected auth error, got: {err_str}"
    );
}

// ── New action tests ──────────────────────────────────────────────────────────

#[tokio::test]
async fn request_retry_ok() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/request/7/retry"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 7,
            "status": 1,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = client.request_retry(7).await.expect("request_retry should succeed");
    assert_eq!(req.id, 7);
}

#[tokio::test]
async fn request_count_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/request/count"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pending": 3,
            "approved": 10,
            "declined": 1,
            "processing": 2,
            "available": 8,
            "total": 24
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let count = client.request_count().await.expect("request_count should succeed");
    assert_eq!(count.pending, 3);
    assert_eq!(count.total, 24);
}

#[tokio::test]
async fn issue_update_ok() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/issue/5/resolved"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 5,
            "issueType": 1,
            "status": 2,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-02T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let issue = client.issue_update(5, "resolved").await.expect("issue_update should succeed");
    assert_eq!(issue.id, 5);
    assert_eq!(issue.status, 2);
}

#[tokio::test]
async fn media_delete_ok() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/media/99"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.media_delete(99).await.expect("media_delete should succeed");
}

#[tokio::test]
async fn media_update_status_ok() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/media/50/available"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.media_update_status(50, "available").await.expect("media_update_status should succeed");
}

#[tokio::test]
async fn user_requests_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/user/1/requests"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pageInfo": null,
            "results": [
                {
                    "id": 11,
                    "status": 2,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let list = client.user_requests(1).await.expect("user_requests should succeed");
    assert_eq!(list.results.len(), 1);
    assert_eq!(list.results[0].id, 11);
}

#[tokio::test]
async fn user_quota_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/user/1/quota"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "movie": { "days": 7, "limit": 5, "used": 2, "remaining": 3 },
            "tv": { "days": 7, "limit": 5, "used": 1, "remaining": 4 }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let quota = client.user_quota(1).await.expect("user_quota should succeed");
    assert!(quota.get("movie").is_some());
}

#[tokio::test]
async fn user_edit_ok() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/user/2"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 2,
            "email": "updated@example.com",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-02-01T00:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let body = serde_json::json!({ "email": "updated@example.com" });
    let user = client.user_edit(2, &body).await.expect("user_edit should succeed");
    assert_eq!(user.id, 2);
    assert_eq!(user.email, "updated@example.com");
}

#[tokio::test]
async fn job_run_ok() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/settings/jobs/plex-recently-added-scan/run"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "plex-recently-added-scan",
            "name": "Plex Recently Added Scan",
            "type": "process",
            "interval": 300,
            "nextExecutionTime": "2024-01-01T01:00:00.000Z",
            "running": true
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.job_run("plex-recently-added-scan").await.expect("job_run should succeed");
    assert_eq!(result["id"], "plex-recently-added-scan");
}

#[tokio::test]
async fn discover_trending_ok() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/discover/trending"))
        .and(header("X-Api-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "page": 1,
            "totalPages": 5,
            "totalResults": 100,
            "results": []
        })))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.discover_trending().await.expect("discover_trending should succeed");
    assert_eq!(result["page"], 1);
}
