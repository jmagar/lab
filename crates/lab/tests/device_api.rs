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
    let app = test_device_router();
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
    let app = test_device_router();
    let response = app
        .oneshot(syslog_request(
            r#"{"device_id":"dookie","events":[{"device_id":"dookie","source":"journald","timestamp_unix_ms":1,"message":"hello","fields":{}}]}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

fn test_device_router() -> axum::Router {
    let state = AppState::new().with_device_store(Arc::new(DeviceFleetStore::default()));
    build_router_with_bearer(state, None, None)
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
