use std::net::IpAddr;

use lab_apis::mcpregistry::types::ListServersParams;
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_runtime_hint ──────────────────────────────────────────────

    #[test]
    fn runtime_hint_allows_valid_runtimes() {
        for hint in ["npx", "uvx", "docker", "dnx", "pipx", "node", "python", "python3", "deno"] {
            assert!(validate_runtime_hint(hint).is_ok(), "expected '{hint}' to be allowed");
        }
    }

    #[test]
    fn runtime_hint_rejects_arbitrary_commands() {
        for hint in ["bash", "sh", "curl", "wget", "perl", "ruby", "php"] {
            assert!(
                validate_runtime_hint(hint).is_err(),
                "expected '{hint}' to be rejected"
            );
        }
    }

    #[test]
    fn runtime_hint_rejects_empty() {
        assert!(validate_runtime_hint("").is_err());
    }

    // ── validate_stdio_argv ────────────────────────────────────────────────

    #[test]
    fn argv_allows_normal_flags() {
        let args = vec!["-y".to_string(), "@scope/pkg@1.0.0".to_string(), "--port".to_string(), "3000".to_string()];
        assert!(validate_stdio_argv(&args).is_ok());
    }

    #[test]
    fn argv_rejects_eval_flag() {
        let args = vec!["--eval".to_string(), "process.exit(1)".to_string()];
        assert!(validate_stdio_argv(&args).is_err());
    }

    #[test]
    fn argv_rejects_short_eval_flag() {
        let args = vec!["-e".to_string(), "bad code".to_string()];
        assert!(validate_stdio_argv(&args).is_err());
    }

    #[test]
    fn argv_rejects_python_command_flag() {
        let args = vec!["-c".to_string(), "import os; os.system('rm -rf /')".to_string()];
        assert!(validate_stdio_argv(&args).is_err());
    }

    #[test]
    fn argv_rejects_require_flag() {
        let args = vec!["--require".to_string(), "malicious".to_string()];
        assert!(validate_stdio_argv(&args).is_err());
    }

    // ── validate_env_var_name ──────────────────────────────────────────────

    #[test]
    fn env_var_name_accepts_valid() {
        for name in ["GITHUB_TOKEN", "FOO", "A1", "SOME_LONG_VAR_123"] {
            assert!(validate_env_var_name(name).is_ok(), "expected '{name}' to be valid");
        }
    }

    #[test]
    fn env_var_name_rejects_lowercase() {
        assert!(validate_env_var_name("github_token").is_err());
    }

    #[test]
    fn env_var_name_rejects_leading_digit() {
        assert!(validate_env_var_name("1FOO").is_err());
    }

    #[test]
    fn env_var_name_rejects_empty() {
        assert!(validate_env_var_name("").is_err());
    }

    #[test]
    fn env_var_name_rejects_special_chars() {
        for name in ["FOO-BAR", "FOO BAR", "FOO.BAR"] {
            assert!(validate_env_var_name(name).is_err(), "expected '{name}' to be rejected");
        }
    }

    // ── validate_env_value ─────────────────────────────────────────────────

    #[test]
    fn env_value_accepts_normal_strings() {
        assert!(validate_env_value("FOO", "ghp_abc123").is_ok());
        assert!(validate_env_value("FOO", "a value with spaces").is_ok());
    }

    #[test]
    fn env_value_rejects_newline() {
        assert!(validate_env_value("FOO", "line1\nline2").is_err());
    }

    #[test]
    fn env_value_rejects_cr() {
        assert!(validate_env_value("FOO", "line1\rline2").is_err());
    }
}
