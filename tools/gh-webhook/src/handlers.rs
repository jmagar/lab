//! Axum handlers for the GitHub webhook receiver.
//!
//! Pipeline for `POST /webhook`:
//!   1. Verify HMAC-SHA256 over the raw body (401 on failure)
//!   2. Dedup by `X-GitHub-Delivery` (200 "duplicate" on repeat)
//!   3. `ping` → 200 "pong"
//!   4. Parse typed `Event` and dispatch:
//!      - `PrComment`   → debouncer.hit()                 → 200 "accepted"
//!      - `PrLifecycle` → append JSONL line               → 200 "accepted"
//!      - `CiFailed`    → append JSONL line               → 200 "accepted"
//!      - `Ignored`     → 200 "ignored"
//!
//! Errors inside the handler never surface secrets. Parse failures become
//! 400 "bad payload" with a `warn!` on the server side.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::config::Config;
use crate::debounce::{Debouncer, PrKey};
use crate::dedup::DedupCache;
use crate::events::{Event, parse_event};
use crate::hmac::verify_signature;
use crate::jsonl::{NotificationLine, append_line};

/// Shared application state wired into every request by `axum::with_state`.
///
/// `Flusher` is intentionally absent: it is owned by the debouncer's flush
/// closure, and handlers never call it directly.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub dedup: Arc<Mutex<DedupCache>>,
    pub debouncer: Arc<Debouncer>,
    pub jsonl_path: PathBuf,
}

/// Build the full axum router for the webhook server.
///
/// Layers (outer → inner): tracing, request body limit, state.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/webhook", post(webhook))
        .route("/healthz", get(healthz))
        // 25 MB matches GitHub's documented maximum webhook payload size.
        .layer(RequestBodyLimitLayer::new(25 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn webhook(
    State(s): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let sig = match headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
    {
        Some(v) => v,
        None => {
            warn!(target: "gh_webhook::handlers", "missing x-hub-signature-256");
            return (StatusCode::UNAUTHORIZED, "missing signature").into_response();
        }
    };
    if verify_signature(s.config.webhook_secret.as_bytes(), &body, sig).is_err() {
        warn!(target: "gh_webhook::handlers", "hmac verification failed");
        return (StatusCode::UNAUTHORIZED, "bad signature").into_response();
    }

    let delivery = headers
        .get("x-github-delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let event = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !delivery.is_empty() {
        // Lock scope is tight — no `.await` while holding the guard.
        let seen = s.dedup.lock().unwrap().seen(delivery);
        if seen {
            info!(target: "gh_webhook::handlers", delivery, "duplicate delivery ignored");
            return (StatusCode::OK, "duplicate").into_response();
        }
    }

    if event == "ping" {
        return (StatusCode::OK, "pong").into_response();
    }

    let parsed = match parse_event(event, &body) {
        Ok(e) => e,
        Err(e) => {
            warn!(target: "gh_webhook::handlers", error = %e, event, "parse error");
            return (StatusCode::BAD_REQUEST, "bad payload").into_response();
        }
    };

    match parsed {
        Event::PrComment {
            owner,
            repo,
            pr,
            branch: _,
        } => {
            s.debouncer
                .hit(PrKey {
                    owner: owner.clone(),
                    repo: repo.clone(),
                    pr,
                })
                .await;
            info!(
                target: "gh_webhook::handlers",
                owner = %owner, repo = %repo, pr, "pr_comment enqueued"
            );
            (StatusCode::OK, "accepted").into_response()
        }
        Event::PrLifecycle {
            owner,
            repo,
            pr,
            branch,
            action,
        } => {
            let display = format!("[PR] {action} {owner}/{repo}#{pr} ({branch})");
            let line = NotificationLine::PrLifecycle {
                owner,
                repo,
                pr,
                branch,
                action,
                display,
            };
            if let Err(e) = append_line(&s.jsonl_path, &line) {
                warn!(target: "gh_webhook::handlers", error = %e, "append pr_lifecycle failed");
            }
            (StatusCode::OK, "accepted").into_response()
        }
        Event::CiFailed {
            owner,
            repo,
            branch,
            url,
            name,
        } => {
            let display = format!("[FAIL] {name} for {owner}/{repo} {branch} - {url}");
            let line = NotificationLine::CiFailed {
                owner,
                repo,
                branch,
                workflow: name,
                run_url: url,
                display,
            };
            if let Err(e) = append_line(&s.jsonl_path, &line) {
                warn!(target: "gh_webhook::handlers", error = %e, "append ci_failed failed");
            }
            (StatusCode::OK, "accepted").into_response()
        }
        Event::Ignored => (StatusCode::OK, "ignored").into_response(),
    }
}
