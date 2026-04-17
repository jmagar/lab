use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use lab::{
    api::{router::build_router_with_bearer, state::AppState},
    device::store::DeviceFleetStore,
};
use tower::ServiceExt;

#[tokio::test]
async fn hello_endpoint_updates_master_store() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(hello_request(
            r#"{"device_id":"dookie","role":"non-master","version":"1.0.0"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn hello_endpoint_normalizes_device_id_before_storage() {
    let (app, store) = test_device_router();
    let response = app
        .oneshot(hello_request(
            r#"{"device_id":"  dookie  ","role":"non-master","version":"1.0.0"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(store.device("dookie").await.is_some());
}

#[tokio::test]
async fn syslog_batch_endpoint_accepts_normalized_events() {
    let (app, store) = test_device_router();
    let response = app
        .oneshot(syslog_request(
            r#"{"device_id":"dookie","events":[{"device_id":"dookie","source":"journald","timestamp_unix_ms":1,"message":"hello","fields":{}}]}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let snapshot = store.device("dookie").await.unwrap();
    assert_eq!(snapshot.logs.len(), 1);
}

#[tokio::test]
async fn get_device_rejects_invalid_device_id() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/device/devices/%20%20%20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn syslog_batch_endpoint_rejects_invalid_device_id() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(syslog_request(
            r#"{"device_id":"   ","events":[{"device_id":"dookie","source":"journald","timestamp_unix_ms":1,"message":"hello","fields":{}}]}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn syslog_batch_endpoint_rejects_mismatched_event_device_id() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(syslog_request(
            r#"{"device_id":"dookie","events":[{"device_id":"tootie","source":"journald","timestamp_unix_ms":1,"message":"hello","fields":{}}]}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn device_oauth_route_calls_runtime_wrapper() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(oauth_relay_start_request(
            r#"{"bind_addr":"127.0.0.1:0","target_url":"http://127.0.0.1:9/callback","request_timeout_ms":100}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["ok"], true);
    assert_ne!(payload["bind_addr"], "127.0.0.1:0");
}

#[tokio::test]
async fn device_oauth_route_rejects_non_loopback_bind_addr() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(oauth_relay_start_request(
            r#"{"bind_addr":"10.0.0.5:9876","target_url":"http://127.0.0.1:9/callback","request_timeout_ms":100}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn device_oauth_route_rejects_invalid_target_url() {
    let (app, _store) = test_device_router();
    let response = app
        .oneshot(oauth_relay_start_request(
            r#"{"bind_addr":"127.0.0.1:0","target_url":"not-a-url","request_timeout_ms":100}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn existing_fleet_logs_search_still_works() {
    let (app, store) = test_device_router();
    store
        .record_hello(lab::device::checkin::DeviceHello {
            device_id: "dookie".to_string(),
            role: "non-master".to_string(),
            version: "1.0.0".to_string(),
        })
        .await;
    store
        .record_logs(
            "dookie",
            vec![lab::device::log_event::DeviceLogEvent {
                device_id: "dookie".to_string(),
                timestamp_unix_ms: 1,
                source: "journald".to_string(),
                level: Some("info".to_string()),
                message: "hello from fleet search".to_string(),
                fields: serde_json::Map::new(),
            }],
        )
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/device/logs/search")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "device_id":"dookie",
                        "query":"hello"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let events: Vec<lab::device::log_event::DeviceLogEvent> =
        serde_json::from_slice(&body).unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].message.contains("fleet search"));
}

fn test_device_router() -> (axum::Router, Arc<DeviceFleetStore>) {
    let store = Arc::new(DeviceFleetStore::default());
    let state = AppState::new().with_device_store(Arc::clone(&store));
    (build_router_with_bearer(state, None, None), store)
}

fn hello_request(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/device/hello")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn syslog_request(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/device/syslog/batch")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}

fn oauth_relay_start_request(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/device/oauth/relay/start")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap()
}
