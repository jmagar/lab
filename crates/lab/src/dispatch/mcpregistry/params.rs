use std::net::IpAddr;

use lab_apis::mcpregistry::types::ListServersParams;
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Extract `server.list` params from the dispatch params object.
pub fn list_servers_params(params: &Value) -> Result<ListServersParams, ToolError> {
    Ok(ListServersParams {
        search: params["search"].as_str().map(str::to_string),
        limit: params["limit"].as_u64().map(|v| v as u32),
        cursor: params["cursor"].as_str().map(str::to_string),
        version: params["version"].as_str().map(str::to_string),
        updated_since: params["updated_since"].as_str().map(str::to_string),
    })
}

/// Validate that a URL from the registry is safe to use as a gateway upstream.
///
/// Rejects non-HTTPS schemes and hosts that resolve to RFC1918, loopback, or
/// link-local addresses (SSRF protection).
///
/// # Blocking
/// Uses synchronous `ToSocketAddrs` DNS — must be called via `spawn_blocking`.
pub fn validate_registry_url(url: &str) -> Result<(), ToolError> {
    use std::net::ToSocketAddrs;

    let ssrf_err = |msg: String| ToolError::Sdk {
        sdk_kind: "ssrf_blocked".to_string(),
        message: msg,
    };

    let parsed = url::Url::parse(url).map_err(|_| ToolError::Sdk {
        sdk_kind: "invalid_param".to_string(),
        message: format!("invalid registry URL: {url}"),
    })?;

    if parsed.scheme() != "https" {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "registry URL must use HTTPS, got '{}': {url}",
                parsed.scheme()
            ),
        });
    }

    let host = parsed.host_str().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "invalid_param".to_string(),
        message: format!("registry URL has no host: {url}"),
    })?;

    let port = parsed.port_or_known_default().unwrap_or(443);

    if let Ok(addr) = host.parse::<IpAddr>() {
        return check_ip_not_private(addr, url);
    }

    let addrs: Vec<_> = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|e| ssrf_err(format!("failed to resolve host '{host}': {e}")))?
        .collect();

    for sock_addr in addrs {
        check_ip_not_private(sock_addr.ip(), url)?;
    }

    Ok(())
}

fn check_ip_not_private(ip: IpAddr, url: &str) -> Result<(), ToolError> {
    let blocked = match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || o[0] == 10
                || (o[0] == 172 && (16..=31).contains(&o[1]))
                || (o[0] == 192 && o[1] == 168)
                || (o[0] == 169 && o[1] == 254)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // fc00::/7 ULA
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || v6.to_ipv4_mapped().is_some_and(|v4| {
                    let o = v4.octets();
                    v4.is_loopback()
                        || o[0] == 10
                        || (o[0] == 172 && (16..=31).contains(&o[1]))
                        || (o[0] == 192 && o[1] == 168)
                        || (o[0] == 169 && o[1] == 254)
                })
        }
    };
    if blocked {
        return Err(ToolError::Sdk {
            sdk_kind: "ssrf_blocked".to_string(),
            message: format!(
                "registry URL resolves to a private/loopback address — blocked to prevent SSRF: {url}"
            ),
        });
    }
    Ok(())
}

/// Extract a required `name` string param.
pub fn require_name(params: &Value) -> Result<String, ToolError> {
    match params["name"].as_str() {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        Some(_) | None => Err(ToolError::MissingParam {
            message: "missing required parameter `name`".to_string(),
            param: "name".to_string(),
        }),
    }
}
