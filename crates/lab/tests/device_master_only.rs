use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use lab::{
    api::{router::build_router_with_bearer, state::AppState},
    config::DeviceRole,
    device::store::DeviceFleetStore,
};
use tower::ServiceExt;

#[tokio::test]
async fn non_master_router_rejects_gateway_api_surface() {
    let app = test_non_master_router();
    let response = app.oneshot(gateway_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn non_master_router_rejects_web_ui_surface() {
    let app = test_non_master_router();
    let response = app.oneshot(web_request()).await.unwrap();
    assert!(matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::FORBIDDEN
    ));
}

fn test_non_master_router() -> axum::Router {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "<html><body>Labby</body></html>").unwrap();

    let state = AppState::new()
        .with_device_store(Arc::new(DeviceFleetStore::default()))
        .with_device_role(DeviceRole::NonMaster)
        .with_web_assets_dir(dir.path().to_path_buf());
    build_router_with_bearer(state, None, None)
}

fn gateway_request() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/gateway")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"action":"gateway.list","params":{}}"#.to_string()))
        .unwrap()
}

fn web_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/gateways/")
        .body(Body::empty())
        .unwrap()
}
