//! gh-webhook server entrypoint.
//!
//! Loads config from env, tightens umask, wires the flusher/debouncer/dedup
//! pipeline, binds an axum listener, and serves until SIGINT/SIGTERM — at
//! which point pending debounced PRs are drained before exit.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

use gh_webhook::config::Config;
use gh_webhook::debounce::{Debouncer, PrKey};
use gh_webhook::dedup::DedupCache;
use gh_webhook::flush::Flusher;
use gh_webhook::github::GithubClient;
use gh_webhook::handlers::{AppState, build_router};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    // Tighten umask so any file we create without an explicit mode defaults
    // to 0600 (owner-only). The jsonl/digest writers set the mode explicitly,
    // but this is belt-and-braces for anything added later.
    // SAFETY: single-threaded startup; no other thread is racing on process
    // state. `libc::umask` is async-signal-safe.
    #[cfg(unix)]
    unsafe {
        libc::umask(0o077);
    }

    let cfg = Config::from_env()?;
    std::fs::create_dir_all(&cfg.data_dir).context("create data dir")?;
    info!(target: "gh_webhook", config = ?cfg, "starting");

    let jsonl_path: PathBuf = cfg.data_dir.join("notifications.jsonl");

    let gh = GithubClient::new("https://api.github.com".into(), cfg.github_token.clone())
        .context("construct github client")?;
    let flusher = Arc::new(Flusher::new(gh, cfg.data_dir.clone(), jsonl_path.clone()));

    let flusher_ref = flusher.clone();
    let debouncer = Arc::new(Debouncer::new(
        Duration::from_secs(cfg.debounce_secs),
        move |k: PrKey, n: u32| {
            let f = flusher_ref.clone();
            async move {
                f.flush_pr(k, n).await;
                Ok(())
            }
        },
    ));

    let state = AppState {
        config: Arc::new(cfg),
        dedup: Arc::new(Mutex::new(DedupCache::new(4096))),
        debouncer: debouncer.clone(),
        jsonl_path,
    };
    let bind = state.config.bind;

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    info!(target: "gh_webhook", addr = %bind, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(debouncer))
        .await
        .context("axum serve")?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("GH_WEBHOOK_LOG")
        .unwrap_or_else(|_| EnvFilter::new("gh_webhook=info,tower_http=info"));
    if std::env::var("GH_WEBHOOK_LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}

async fn shutdown_signal(debouncer: Arc<Debouncer>) {
    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let term = async {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    info!(target: "gh_webhook", "shutting down, draining debouncer");
    debouncer.drain().await;
    info!(target: "gh_webhook", "drain complete");
}
