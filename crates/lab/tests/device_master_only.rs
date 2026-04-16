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
use lab_auth::config::{AuthConfig, AuthMode, GoogleConfig};
use tower::ServiceExt;

#[tokio::test]
async fn non_master_router_rejects_gateway_api_surface() {
    let fixture = test_non_master_router();
    let app = fixture.router;
    let response = app.oneshot(gateway_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn non_master_router_rejects_web_ui_surface() {
    let fixture = test_non_master_router();
    let app = fixture.router;
    let response = app.oneshot(web_request()).await.unwrap();
    assert!(matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::FORBIDDEN
    ));
}

#[tokio::test]
async fn non_master_router_does_not_mount_oauth_metadata_surface() {
    let auth_state = test_lab_auth_state().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.html"),
        "<html><body>Labby</body></html>",
    )
    .unwrap();

    let state = AppState::new()
        .with_device_store(Arc::new(DeviceFleetStore::default()))
        .with_device_role(DeviceRole::NonMaster)
        .with_web_assets_dir(dir.path().to_path_buf());
    let app = lab::api::router::build_router(state, None, Some(auth_state), None, &[]);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/.well-known/oauth-authorization-server")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(matches!(
        response.status(),
        StatusCode::NOT_FOUND | StatusCode::FORBIDDEN
    ));
}

struct NonMasterRouterFixture {
    #[allow(dead_code)]
    dir: tempfile::TempDir,
    router: axum::Router,
}

fn test_non_master_router() -> NonMasterRouterFixture {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("index.html"),
        "<html><body>Labby</body></html>",
    )
    .unwrap();

    let state = AppState::new()
        .with_device_store(Arc::new(DeviceFleetStore::default()))
        .with_device_role(DeviceRole::NonMaster)
        .with_web_assets_dir(dir.path().to_path_buf());
    NonMasterRouterFixture {
        dir,
        router: build_router_with_bearer(state, None, None),
    }
}

fn gateway_request() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/gateway")
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"action":"gateway.list","params":{}}"#.to_string(),
        ))
        .unwrap()
}

fn web_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/gateways/")
        .body(Body::empty())
        .unwrap()
}

async fn test_lab_auth_state() -> lab_auth::state::AuthState {
    let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
    let config = AuthConfig {
        mode: AuthMode::OAuth,
        public_url: Some(url::Url::parse("https://lab.example.com").unwrap()),
        sqlite_path: dir.path().join("auth.db"),
        key_path: dir.path().join("auth-jwt.pem"),
        bootstrap_secret: Some("bootstrap-secret".to_string()),
        google: GoogleConfig {
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            callback_path: "/auth/google/callback".to_string(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        },
        ..AuthConfig::default()
    };
    lab_auth::state::AuthState::new(config).await.unwrap()
}
