// Dev mockup file server — serves self-contained HTML from ~/.superpowers/brainstorm/content/.
//
// Routes registered in router.rs:
//   GET /dev          → newest .html file (brainstorm scratch pad)
//   GET /dev/setup    → newest .html whose stem contains "setup"
//   GET /dev/settings → newest .html whose stem contains "settings"
//
// Files are read from disk on every request — no restart needed when mockups change.
// This module lives separately from web.rs so it is not affected by web.rs refactors.
// DO NOT make these handlers delegate to serve_web_request — that serves the Next.js
// SPA and would break the mockup routes.

use std::path::PathBuf;
use std::time::SystemTime;

use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;

fn content_dir() -> PathBuf {
    // Use $HOME so the path is stable regardless of process CWD.
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".superpowers/brainstorm/content"))
        .unwrap_or_else(|| PathBuf::from(".superpowers/brainstorm/content"))
}

// Returns the newest .html file whose stem contains `fragment`.
// When `fragment` is None, returns the newest file overall.
fn newest(fragment: Option<&str>) -> Option<PathBuf> {
    std::fs::read_dir(content_dir()).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("html"))
        .filter(|e| {
            fragment.is_none_or(|n| {
                e.path().file_stem().and_then(|s| s.to_str()).is_some_and(|s| s.contains(n))
            })
        })
        .filter_map(|e| {
            e.metadata().ok()
                .and_then(|m| m.modified().ok())
                .map(|t: SystemTime| (e.path(), t))
        })
        .max_by_key(|(_, t)| *t)
        .map(|(p, _)| p)
}

fn respond(fragment: Option<&str>) -> Response {
    match newest(fragment) {
        None => Html(format!(
            "<p style='font-family:sans-serif;padding:2rem'>No{} HTML file found in \
             <code>~/.superpowers/brainstorm/content/</code></p>",
            fragment.map(|n| format!(" '{n}'")).unwrap_or_default()
        )).into_response(),
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "failed to read dev mockup");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
    }
}

// GET /dev and /dev/
pub async fn serve() -> Response { respond(None) }

// GET /dev/:name and /dev/:name/ — e.g. /dev/setup → setup.html
pub async fn serve_named(axum::extract::Path(name): axum::extract::Path<String>) -> Response {
    // Reject traversal — name must be a plain identifier, no separators or dots.
    if name.contains('/') || name.contains('\\') || name.contains('.') {
        return StatusCode::NOT_FOUND.into_response();
    }
    respond(Some(&name))
}
