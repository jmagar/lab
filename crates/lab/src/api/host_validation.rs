//! Host header validation Layer.
//!
//! v1 ships without OAuth (per locked Q5). DNS rebinding mitigation: any
//! request whose `Host` header is not a loopback name (`localhost`,
//! `127.0.0.1`, `[::1]`) — optionally with a port suffix — is rejected
//! with 421 Misdirected Request before it reaches the dispatch layer.
//!
//! Apply this layer to the `/v1/setup/*` and `/v1/dispatch/*` route groups.
//! It is intentionally conservative: a missing `Host` header is rejected
//! too (no browser-driven request omits it).
//!
//! Bypass for tests: set the `LAB_HOST_VALIDATION_DISABLED=1` env var.

use axum::{
    body::Body,
    http::{Request, Response, StatusCode, header::HOST},
    middleware::Next,
};

/// Loopback hostnames accepted by the validator.
const LOOPBACK_HOSTS: &[&str] = &["127.0.0.1", "localhost", "::1"];

/// Returns `true` if `host_value` (e.g. `"localhost:8765"` or `"[::1]"`)
/// resolves to a loopback hostname.
#[must_use]
pub fn is_loopback_host_value(host_value: &str) -> bool {
    let trimmed = host_value.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Strip optional port suffix.
    // Handle IPv6 with brackets first: `[::1]:8765` or `[::1]`.
    let stripped: &str = if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            &rest[..end]
        } else {
            return false;
        }
    } else if let Some((host, _port)) = trimmed.rsplit_once(':')
        && !host.contains(':')
    {
        host
    } else {
        trimmed
    };
    LOOPBACK_HOSTS.contains(&stripped)
}

/// Axum middleware function. Use with
/// `Router::layer(axum::middleware::from_fn(host_validation_layer))`.
pub async fn host_validation_layer(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    if std::env::var("LAB_HOST_VALIDATION_DISABLED").as_deref() == Ok("1") {
        return Ok(next.run(req).await);
    }
    let host = req
        .headers()
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if !is_loopback_host_value(host) {
        tracing::warn!(
            surface = "api",
            kind = "host_validation_failed",
            host = %host,
            path = %req.uri().path(),
            "rejecting non-loopback Host header"
        );
        return Err(StatusCode::MISDIRECTED_REQUEST);
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_loopback_names() {
        assert!(is_loopback_host_value("localhost"));
        assert!(is_loopback_host_value("127.0.0.1"));
        assert!(is_loopback_host_value("[::1]"));
        assert!(is_loopback_host_value("localhost:8765"));
        assert!(is_loopback_host_value("127.0.0.1:8765"));
        assert!(is_loopback_host_value("[::1]:8765"));
    }

    #[test]
    fn rejects_non_loopback() {
        assert!(!is_loopback_host_value("evil.example.com"));
        assert!(!is_loopback_host_value("192.168.1.5:8765"));
        assert!(!is_loopback_host_value(""));
        assert!(!is_loopback_host_value("attacker.local"));
    }

    #[test]
    fn rejects_malformed_ipv6() {
        // No closing bracket — reject defensively.
        assert!(!is_loopback_host_value("[::1"));
    }
}
