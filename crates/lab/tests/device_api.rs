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
async fn syslog_batch_endpoint_accepts_normalized_events() {
    let (app, store) = test_device_router();
    let response = app
        .oneshot(syslog_request(
            r#"{"device_id":"dookie","events":[{"device_id":"dookie","source":"journald","timestamp_unix_ms":1,"message":"hello","fields":{}}]}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(store.logs_for_device("dookie").await.len(), 1);
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
