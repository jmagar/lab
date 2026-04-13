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
