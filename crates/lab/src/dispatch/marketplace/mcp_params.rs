//! Param coercion and security guards for `mcp.*` actions in the marketplace dispatch.
//!
//! Security guards (SSRF, argv allowlist) are copied VERBATIM from
//! `dispatch/mcpregistry/params.rs` — do NOT weaken them.
//!
//! TODO(M1): DNS rebinding gap — SSRF guards check IP at validation time but DNS
//! resolution happens at connect time. A hostname that resolves to an RFC1918 address
//! after validation bypasses these guards. Full mitigation requires IP-at-connect-time
//! validation intercepting at the reqwest layer. Deferred.

use std::net::IpAddr;

#[cfg(feature = "mcpregistry")]
use lab_apis::mcpregistry::types::{
    LabRegistryMetadata, LabRegistrySetupDifficulty, LabRegistryTransportScore, ListServersParams,
};
use serde_json::Value;

use crate::dispatch::error::ToolError;

/// Runtime hints the registry is allowed to produce and we are willing to execute.
const ALLOWED_RUNTIME_HINTS: &[&str] = &[
    "npx", "uvx", "docker", "dnx", "pipx", "node", "python", "python3", "deno",
];

/// Argv flags that can execute arbitrary code strings when passed to any allowed runtime.
/// These are blocked regardless of which runtimeHint is used.
const DANGEROUS_ARGV_FLAGS: &[&str] = &[
    "--eval",    // node, deno: evaluate a JS string
    "-e",        // node: short for --eval
    "--exec",    // various
    "--stdin",   // node: read script from stdin
    "-c",        // python/shell: execute a command string
    "--require", // node: pre-require a module before the main script
    "-r",        // node: short for --require
    "--import",  // node 20+: unconditional ES module import before script
];

/// Validate a `runtimeHint` string against the static allowlist.
///
/// Returns an error if the hint is not in [`ALLOWED_RUNTIME_HINTS`].
pub fn validate_runtime_hint(hint: &str) -> Result<(), ToolError> {
    if ALLOWED_RUNTIME_HINTS.contains(&hint) {
        Ok(())
    } else {
        Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "runtimeHint '{hint}' is not in the allowed list; must be one of: {}",
                ALLOWED_RUNTIME_HINTS.join(", ")
            ),
        })
    }
}

/// Validate that none of the argv strings is a dangerous code-execution flag.
///
/// Checks both exact matches and `--flag=value` prefix forms.
pub fn validate_stdio_argv(args: &[String]) -> Result<(), ToolError> {
    for arg in args {
        let normalized = arg.split('=').next().unwrap_or(arg);
        if DANGEROUS_ARGV_FLAGS.contains(&normalized) {
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: format!(
                    "argv flag '{arg}' is not allowed — it can execute arbitrary code"
                ),
            });
        }
    }
    Ok(())
}

/// Validate an environment variable name: must match `^[A-Z][A-Z0-9_]*$`.
pub fn validate_env_var_name(name: &str) -> Result<(), ToolError> {
    let valid = !name.is_empty()
        && name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');

    if valid {
        Ok(())
    } else {
        Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "env var name '{name}' is invalid; must match ^[A-Z][A-Z0-9_]*$"
            ),
        })
    }
}

/// Validate an environment variable value: must not contain embedded newlines.
pub fn validate_env_value(key: &str, value: &str) -> Result<(), ToolError> {
    if value.contains('\n') || value.contains('\r') {
        Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("env var '{key}' value must not contain newlines"),
        })
    } else {
        Ok(())
    }
}

/// Resolve the effective `search` string from `search` + `owner` dispatch params.
///
/// Precedence:
/// 1. explicit `search` wins if present (owner is silently ignored).
/// 2. `owner` is validated (non-empty after trim, no `/`, no whitespace) and
///    synthesized to `io.github.{owner}/` lowercased.
/// 3. invalid `owner` returns an `invalid_param` error so callers see the
///    problem instead of falling through to an unfiltered list.
///
/// The registry API has no structured owner field — this is a client-side
/// convenience only and does not match non-GitHub publishers.
pub fn resolve_search_for_rest(
    search: Option<&str>,
    owner: Option<&str>,
) -> Result<Option<String>, ToolError> {
    if let Some(s) = search {
        return Ok(Some(s.to_string()));
    }
    let Some(raw) = owner else {
        return Ok(None);
    };
    let owner = raw.trim();
    if owner.is_empty() {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: "`owner` must not be empty".to_string(),
        });
    }
    if owner.chars().any(|c| c == '/' || c.is_whitespace()) {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: "`owner` must be a bare GitHub username/org (no slashes or whitespace)"
                .to_string(),
        });
    }
    Ok(Some(format!("io.github.{}/", owner.to_ascii_lowercase())))
}

fn optional_string_param<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_) => Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("`{key}` must be a string"),
        }),
    }
}

pub fn resolve_search(params: &Value) -> Result<Option<String>, ToolError> {
    resolve_search_for_rest(
        optional_string_param(params, "search")?,
        optional_string_param(params, "owner")?,
    )
}

fn optional_bool_param(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("`{key}` must be a boolean"),
        }),
    }
}

/// Extract `mcp.list` params from the dispatch params object.
#[cfg(feature = "mcpregistry")]
pub fn list_servers_params(params: &Value) -> Result<ListServersParams, ToolError> {
    Ok(ListServersParams {
        search: resolve_search(params)?,
        limit: params["limit"].as_u64().map(|v| v as u32),
        cursor: params["cursor"].as_str().map(str::to_string),
        version: params["version"].as_str().map(str::to_string),
        updated_since: params["updated_since"].as_str().map(str::to_string),
        featured: optional_bool_param(params, "featured")?,
        reviewed: optional_bool_param(params, "reviewed")?,
        recommended: optional_bool_param(params, "recommended")?,
        hidden: optional_bool_param(params, "hidden")?,
        tag: optional_string_param(params, "tag")?.map(str::to_string),
    })
}

#[cfg(feature = "mcpregistry")]
pub fn parse_lab_metadata(value: &Value) -> Result<LabRegistryMetadata, ToolError> {
    let metadata: LabRegistryMetadata = serde_json::from_value(value.clone()).map_err(|e| {
        ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("invalid Lab metadata: {e}"),
        }
    })?;
    validate_lab_metadata(&metadata)?;
    Ok(normalize_lab_metadata(metadata))
}

#[cfg(feature = "mcpregistry")]
fn validate_lab_metadata(metadata: &LabRegistryMetadata) -> Result<(), ToolError> {
    if metadata.audit.is_some() {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: "`audit` is managed by Lab and cannot be set manually".to_string(),
        });
    }

    if let Some(curation) = &metadata.curation {
        for tag in &curation.tags {
            if tag.trim().is_empty() {
                return Err(invalid_metadata("curation.tags must not contain empty values"));
            }
        }
    }
    if let Some(trust) = &metadata.trust {
        validate_timestamp(trust.reviewed_at.as_deref(), "trust.reviewed_at")?;
    }
    if let Some(quality) = &metadata.quality {
        validate_timestamp(
            quality.last_install_tested_at.as_deref(),
            "quality.last_install_tested_at",
        )?;
        match quality.transport_score {
            Some(LabRegistryTransportScore::Good | LabRegistryTransportScore::Mixed | LabRegistryTransportScore::Poor) | None => {}
        }
    }
    if let Some(ux) = &metadata.ux {
        match ux.setup_difficulty {
            Some(LabRegistrySetupDifficulty::Easy | LabRegistrySetupDifficulty::Medium | LabRegistrySetupDifficulty::Hard) | None => {}
        }
    }
    Ok(())
}

#[cfg(feature = "mcpregistry")]
fn normalize_lab_metadata(mut metadata: LabRegistryMetadata) -> LabRegistryMetadata {
    if let Some(curation) = metadata.curation.as_mut() {
        curation.tags = curation
            .tags
            .iter()
            .map(|tag| tag.trim())
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        curation.notes = curation
            .notes
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
    }
    if let Some(trust) = metadata.trust.as_mut() {
        trust.reviewed_at = normalize_optional_string(trust.reviewed_at.take());
    }
    if let Some(quality) = metadata.quality.as_mut() {
        quality.last_install_tested_at = normalize_optional_string(quality.last_install_tested_at.take());
    }
    metadata
}

#[cfg(feature = "mcpregistry")]
fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(feature = "mcpregistry")]
fn validate_timestamp(value: Option<&str>, field: &str) -> Result<(), ToolError> {
    let Some(value) = value else {
        return Ok(());
    };
    value.parse::<jiff::Timestamp>().map_err(|_| invalid_metadata(&format!(
        "`{field}` must be an RFC3339 timestamp"
    )))?;
    Ok(())
}

#[cfg(feature = "mcpregistry")]
fn invalid_metadata(message: &str) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "invalid_param".to_string(),
        message: message.to_string(),
    }
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
                // 100.64.0.0/10 — Tailscale/CGNAT shared address space.
                // On a Tailscale node these addresses reach internal services.
                || (o[0] == 100 && (64..=127).contains(&o[1]))
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
                        || (o[0] == 100 && (64..=127).contains(&o[1]))
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
