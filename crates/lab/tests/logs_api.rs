use std::pin::pin;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};

use axum::{
    Router,
    body::{Body, HttpBody},
    http::{Request, StatusCode, header},
};
use futures::future::poll_fn;
use tower::ServiceExt;

static LOG_SYSTEM_TEST_LOCK: Mutex<()> = Mutex::new(());

fn lock_log_system_tests() -> MutexGuard<'static, ()> {
    LOG_SYSTEM_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn raw_gateway_event(message: &str) -> lab::dispatch::logs::types::RawLogEvent {
    lab::dispatch::logs::types::RawLogEvent {
        ts: Some(1_713_225_600_000),
        level: Some("warn".to_string()),
        subsystem: Some("gateway".to_string()),
        surface: Some("api".to_string()),
        action: Some("gateway.list".to_string()),
        message: message.to_string(),
        request_id: Some("req-gateway".to_string()),
        session_id: None,
        correlation_id: None,
        trace_id: None,
        span_id: None,
        instance: Some("default".to_string()),
        auth_flow: None,
        outcome_kind: Some("ok".to_string()),
        fields_json: serde_json::json!({"route":"gateway.list"}),
        source_kind: None,
        source_node_id: None,
        source_device_id: None,
        ingest_path: None,
        upstream_event_id: None,
    }
}

async fn test_app() -> (Router, Arc<lab::dispatch::logs::types::LogSystem>) {
    let logs_system = lab::dispatch::logs::client::bootstrap_running_log_system_for_test(16)
        .await
        .expect("log system");
    let state = lab::api::state::AppState::new().with_log_system(Arc::clone(&logs_system));
    (
        lab::api::router::build_router_with_bearer(state, None, None),
        logs_system,
    )
}

async fn read_next_body_chunk(body: Body) -> axum::body::Bytes {
    let mut body = pin!(body);
    loop {
        let frame = poll_fn(|cx| body.as_mut().poll_frame(cx))
            .await
            .expect("body frame result")
            .expect("body frame available");
        if let Ok(data) = frame.into_data() {
            return data;
        }
    }
}

#[tokio::test]
async fn post_logs_search_route_exists() {
    let _lock = lock_log_system_tests();
    let (app, _logs_system) = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({"action":"logs.search","params":{"query":{}}}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn logs_stream_sse_route_emits_event_stream_content_type() {
    let _lock = lock_log_system_tests();
    let (app, _logs_system) = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/logs/stream")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
}

#[tokio::test]
async fn logs_sse_reconnect_resumes_stream() {
    let _lock = lock_log_system_tests();
    let (app, logs_system) = test_app().await;

    let first_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/logs/stream")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("first response");

    logs_system
        .ingest(raw_gateway_event("first sse payload"))
        .await
        .expect("first event");

    let first_chunk = read_next_body_chunk(first_response.into_body()).await;
    let first_text = String::from_utf8(first_chunk.to_vec()).expect("utf8 body");
    assert!(first_text.contains("first sse payload"));

    let second_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/logs/stream")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("second response");

    logs_system
        .ingest(raw_gateway_event("second sse payload"))
        .await
        .expect("second event");

    let second_chunk = read_next_body_chunk(second_response.into_body()).await;
    let second_text = String::from_utf8(second_chunk.to_vec()).expect("utf8 body");
    assert!(second_text.contains("second sse payload"));
    lab::dispatch::logs::client::clear_installed_log_system_for_test();
}

#[tokio::test]
async fn logs_mcp_tail_matches_api_query_semantics() {
    let _lock = lock_log_system_tests();
    let (app, logs_system) = test_app().await;
    logs_system
        .ingest(raw_gateway_event("shared tail semantics"))
        .await
        .expect("seed event");

    let mcp_value = lab::mcp::services::logs::dispatch(
        "logs.tail",
        serde_json::json!({ "after_ts": 0, "limit": 10 }),
    )
    .await
    .expect("mcp tail");

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "action":"logs.tail",
                        "params":{"after_ts":0,"limit":10}
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response bytes");
    let api_value: serde_json::Value = serde_json::from_slice(&body).expect("json body");
    assert_eq!(api_value, mcp_value);
    lab::dispatch::logs::client::clear_installed_log_system_for_test();
}

#[tokio::test]
async fn logs_routes_respect_runtime_service_filtering() {
    let logs_system = lab::dispatch::logs::client::bootstrap_running_log_system_for_test(16)
        .await
        .expect("log system");
    let state = lab::api::state::AppState::from_registry(lab::registry::ToolRegistry::new())
        .with_log_system(logs_system);
    let app = lab::api::router::build_router_with_bearer(state, None, None);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({"action":"logs.search","params":{"query":{}}}).to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
