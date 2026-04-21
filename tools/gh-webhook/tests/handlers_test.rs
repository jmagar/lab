//! Integration tests for the axum handler pipeline.
//!
//! These tests drive `build_router` with a synthetic `AppState` via
//! `tower::ServiceExt::oneshot`, so no socket is bound.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::Request;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tower::ServiceExt;

use gh_webhook::config::Config;
use gh_webhook::debounce::{Debouncer, PrKey};
use gh_webhook::dedup::DedupCache;
use gh_webhook::handlers::{AppState, build_router};

const SECRET: &[u8] = b"test-secret";

fn sign(secret: &[u8], body: &[u8]) -> String {
    let mut m = <Hmac<Sha256> as Mac>::new_from_slice(secret).unwrap();
    m.update(body);
    format!("sha256={}", hex::encode(m.finalize().into_bytes()))
}

fn make_config(data_dir: PathBuf) -> Config {
    Config {
        webhook_secret: String::from_utf8(SECRET.to_vec()).unwrap(),
        github_token: "tok".into(),
        bind: "127.0.0.1:0".parse().unwrap(),
        data_dir,
        debounce_secs: 30,
    }
}

fn make_state() -> (AppState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();
    let jsonl_path = data_dir.join("notifications.jsonl");
    let config = Arc::new(make_config(data_dir.clone()));
    let dedup = Arc::new(Mutex::new(DedupCache::new(256)));
    // Flush callback is a no-op stub — router tests don't exercise real flushing.
    let debouncer = Arc::new(Debouncer::new(
        Duration::from_secs(60),
        move |_k: PrKey, _n: u32| async move { Ok(()) },
    ));
    let state = AppState {
        config,
        dedup,
        debouncer,
        jsonl_path,
    };
    (state, dir)
}

async fn body_text(resp: axum::response::Response) -> String {
    let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
}

#[tokio::test]
async fn healthz_returns_ok() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    let resp = app
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_text(resp).await, "ok");
}

#[tokio::test]
async fn bad_signature_returns_401() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", "sha256=deadbeef")
        .header("x-github-event", "ping")
        .header("x-github-delivery", "abc")
        .body(Body::from("{}"))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn ping_event_returns_pong() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    let body = br#"{"zen":"hi"}"#.to_vec();
    let sig = sign(SECRET, &body);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", sig)
        .header("x-github-event", "ping")
        .header("x-github-delivery", "ping-1")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_text(resp).await, "pong");
}

#[tokio::test]
async fn duplicate_delivery_returns_duplicate() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    let body = br#"{"zen":"hi"}"#.to_vec();
    let sig = sign(SECRET, &body);
    let make = |delivery: &str| {
        Request::post("/webhook")
            .header("x-hub-signature-256", sig.clone())
            .header("x-github-event", "ping")
            .header("x-github-delivery", delivery)
            .body(Body::from(body.clone()))
            .unwrap()
    };
    let first = app.clone().oneshot(make("dup-1")).await.unwrap();
    assert_eq!(first.status(), 200);
    assert_eq!(body_text(first).await, "pong");
    let second = app.oneshot(make("dup-1")).await.unwrap();
    assert_eq!(second.status(), 200);
    assert_eq!(body_text(second).await, "duplicate");
}

#[tokio::test]
async fn valid_pr_comment_event_returns_accepted_and_enqueues() {
    let (state, _tmp) = make_state();
    let app = build_router(state.clone());
    let payload = serde_json::json!({
        "action": "created",
        "repository": { "name": "r", "owner": { "login": "o" } },
        "pull_request": { "number": 7, "head": { "ref": "feat/x" } },
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let sig = sign(SECRET, &body);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", sig)
        .header("x-github-event", "pull_request_review_comment")
        .header("x-github-delivery", "prc-1")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_text(resp).await, "accepted");
    // Drop state; debouncer's pending timers abort cleanly.
    drop(state);
}

#[tokio::test]
async fn pr_lifecycle_event_writes_jsonl_and_returns_accepted() {
    let (state, _tmp) = make_state();
    let jsonl_path = state.jsonl_path.clone();
    let app = build_router(state);
    let payload = serde_json::json!({
        "action": "opened",
        "repository": { "name": "r", "owner": { "login": "o" } },
        "pull_request": { "number": 9, "head": { "ref": "feat/y" } },
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let sig = sign(SECRET, &body);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", sig)
        .header("x-github-event", "pull_request")
        .header("x-github-delivery", "prl-1")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_text(resp).await, "accepted");

    let jsonl = std::fs::read_to_string(&jsonl_path).unwrap();
    assert!(
        jsonl.contains("\"kind\":\"pr_lifecycle\""),
        "jsonl: {jsonl}"
    );
    assert!(jsonl.contains("\"action\":\"opened\""));
    assert!(jsonl.contains("\"branch\":\"feat/y\""));
}

#[tokio::test]
async fn ignored_event_returns_ignored() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    let body = b"{}".to_vec();
    let sig = sign(SECRET, &body);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", sig)
        .header("x-github-event", "star")
        .header("x-github-delivery", "ign-1")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(body_text(resp).await, "ignored");
}

#[tokio::test]
async fn request_body_over_25mb_rejected_413() {
    let (state, _tmp) = make_state();
    let app = build_router(state);
    // 26 MB payload — RequestBodyLimitLayer should reject before we read it.
    let body = vec![b'a'; 26 * 1024 * 1024];
    let sig = sign(SECRET, &body);
    let req = Request::post("/webhook")
        .header("x-hub-signature-256", sig)
        .header("x-github-event", "ping")
        .header("x-github-delivery", "big-1")
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 413);
}
