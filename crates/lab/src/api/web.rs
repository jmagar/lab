use std::path::{Component, Path, PathBuf};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};

use super::state::AppState;
use crate::config::DeviceRole;

fn sanitize_relative_path(path: &str) -> PathBuf {
    let trimmed = path.trim_start_matches('/');
    let mut out = PathBuf::new();
    for component in Path::new(trimmed).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir
            | Component::Prefix(_)
            | Component::RootDir => return PathBuf::new(),
        }
    }
    out
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        Some("map") => "application/json",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn cache_control_for(path: &Path) -> &'static str {
    if path.file_name().and_then(|name| name.to_str()) == Some("index.html") {
        "no-store"
    } else {
        "public, max-age=31536000, immutable"
    }
}

async fn read_asset_file(path: &Path) -> std::io::Result<Vec<u8>> {
    tokio::fs::read(path).await
}

async fn canonicalize_if_exists(path: &Path) -> std::io::Result<PathBuf> {
    tokio::fs::canonicalize(path).await
}

async fn resolve_asset_path(base_dir: &Path, request_path: &str) -> PathBuf {
    let relative = sanitize_relative_path(request_path);
    let candidate = if relative.as_os_str().is_empty() {
        base_dir.join("index.html")
    } else {
        base_dir.join(relative)
    };

    match tokio::fs::metadata(&candidate).await {
        Ok(metadata) if metadata.is_file() => candidate,
        Ok(metadata) if metadata.is_dir() => candidate.join("index.html"),
        _ => base_dir.join("index.html"),
    }
}

pub async fn serve_web_request(State(state): State<AppState>, request: Request) -> Response {
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return StatusCode::NOT_FOUND.into_response();
    }

    if matches!(state.device_role, Some(DeviceRole::NonMaster)) {
        return (
            StatusCode::FORBIDDEN,
            "web ui is disabled on non-master devices",
        )
            .into_response();
    }

    let Some(base_dir) = state.web_assets_dir.as_deref() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let request_path = request.uri().path();
    let resolved = resolve_asset_path(base_dir, request_path).await;

    let canonical_base = match canonicalize_if_exists(base_dir).await {
        Ok(path) => path,
        Err(error) => {
            tracing::warn!(path = %base_dir.display(), error = %error, "failed to canonicalize web assets root");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let canonical_resolved = match canonicalize_if_exists(&resolved).await {
        Ok(path) => path,
        Err(error) => {
            tracing::warn!(path = %resolved.display(), error = %error, "failed to canonicalize web asset");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    if !canonical_resolved.starts_with(&canonical_base) {
        tracing::warn!(
            requested = %resolved.display(),
            resolved = %canonical_resolved.display(),
            base = %canonical_base.display(),
            "rejected web asset request that escaped the asset root"
        );
        return StatusCode::NOT_FOUND.into_response();
    };

    match read_asset_file(&canonical_resolved).await {
        Ok(bytes) => {
            let content_type = guess_content_type(&canonical_resolved);
            let cache_control = cache_control_for(&canonical_resolved);
            let mut response = Response::new(if request.method() == Method::HEAD {
                Body::empty()
            } else {
                Body::from(bytes)
            });
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static(cache_control),
            );
            response
        }
        Err(error) => {
            tracing::warn!(path = %resolved.display(), error = %error, "failed to serve web asset");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
