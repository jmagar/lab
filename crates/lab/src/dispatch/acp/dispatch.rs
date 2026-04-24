//! Top-level action router for the `acp` dispatch service.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;

use crate::acp::types::StartSessionInput;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};

use super::catalog::ACTIONS;
use super::client::require_registry;
use super::params::{opt_str, opt_u64, require_str};

/// SSE ticket lifetime in seconds.
const TICKET_TTL_SECS: u64 = 30;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("acp", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        other => {
            if !ACTIONS.iter().any(|a| a.name == other) {
                return Err(ToolError::UnknownAction {
                    message: format!("unknown action `{other}` for service `acp`"),
                    valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                    hint: None,
                });
            }
            let registry = require_registry()?;
            dispatch_with_registry(&registry, other, params).await
        }
    }
}

pub async fn dispatch_with_registry(
    registry: &crate::acp::registry::AcpSessionRegistry,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("acp", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }

        // ── Provider actions ──────────────────────────────────────────────

        "provider.list" => {
            let health = registry.provider_health();
            to_json(json!({
                "providers": [{
                    "name": health.provider,
                    "available": health.available,
                    "version": health.version,
                    "error": health.message,
                }]
            }))
        }

        "provider.get" => {
            let _provider = require_str(&params, "provider")?;
            // Phase 1: single provider — return health regardless of name.
            let health = registry.provider_health();
            to_json(json!({
                "name": health.provider,
                "available": health.available,
                "version": health.version,
                "error": health.message,
            }))
        }

        "provider.select" => {
            let provider = require_str(&params, "provider")?;
            // Phase 1: stub — only "codex" is supported.
            if provider != "codex" {
                return Err(ToolError::InvalidParam {
                    message: format!("unknown provider `{provider}`; only 'codex' is supported"),
                    param: "provider".to_string(),
                });
            }
            to_json(json!({ "selected": provider }))
        }

        // ── Session read actions ──────────────────────────────────────────

        "session.list" => {
            let principal = opt_str(&params, "principal").unwrap_or("");
            let sessions = registry.list_sessions(principal).await;
            to_json(json!({ "sessions": sessions }))
        }

        "session.get" => {
            let session_id = require_str(&params, "session_id")?;
            let summary = registry.get_session(session_id).await.ok_or_else(|| {
                ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!("session `{session_id}` not found"),
                }
            })?;
            to_json(summary)
        }

        "session.events" => {
            let session_id = require_str(&params, "session_id")?;
            let principal = opt_str(&params, "principal").unwrap_or("");
            let since = opt_u64(&params, "since")?.unwrap_or(0);
            let events = registry
                .get_events_since(session_id, since, principal)
                .await?;
            to_json(json!({ "events": events, "count": events.len() }))
        }

        // ── Session write actions ─────────────────────────────────────────

        "session.start" => {
            let principal = opt_str(&params, "principal").unwrap_or("");
            let title = opt_str(&params, "title").map(|s| s.to_string());
            let cwd = opt_str(&params, "cwd").unwrap_or("").to_string();

            let input = StartSessionInput {
                title,
                cwd,
                principal: if principal.is_empty() {
                    None
                } else {
                    Some(principal.to_string())
                },
            };
            let summary = registry.create_session(input, principal).await?;
            to_json(summary)
        }

        "session.prompt" => {
            let session_id = require_str(&params, "session_id")?;
            let principal = opt_str(&params, "principal").unwrap_or("");
            let text = params
                .get("text")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ToolError::MissingParam {
                    message: "required param `text` is missing or empty".to_string(),
                    param: "text".to_string(),
                })?;

            registry
                .prompt_session(session_id, text, principal)
                .await?;
            to_json(json!({ "ok": true, "session_id": session_id }))
        }

        "session.cancel" => {
            // Destructive — callers must pass `"confirm": true` in params.
            let confirmed = params.get("confirm").and_then(|v| v.as_bool()).unwrap_or(false);
            if !confirmed {
                return Err(ToolError::ConfirmationRequired {
                    message: "session.cancel is destructive; pass `\"confirm\": true` to proceed"
                        .to_string(),
                });
            }
            let session_id = require_str(&params, "session_id")?;
            let principal = opt_str(&params, "principal").unwrap_or("");
            registry.cancel_session(session_id, principal).await?;
            to_json(json!({ "ok": true, "session_id": session_id }))
        }

        "session.close" => {
            // Destructive — callers must pass `"confirm": true` in params.
            let confirmed = params.get("confirm").and_then(|v| v.as_bool()).unwrap_or(false);
            if !confirmed {
                return Err(ToolError::ConfirmationRequired {
                    message: "session.close is destructive; pass `\"confirm\": true` to proceed"
                        .to_string(),
                });
            }
            let session_id = require_str(&params, "session_id")?;
            let principal = opt_str(&params, "principal").unwrap_or("");
            registry.close_session(session_id, principal).await?;
            to_json(json!({ "ok": true, "session_id": session_id }))
        }

        "session.subscribe_ticket" => {
            let session_id = require_str(&params, "session_id")?;
            let principal = opt_str(&params, "principal").unwrap_or("");
            let ticket = issue_subscribe_ticket(session_id, principal)?;
            to_json(json!({
                "ticket": ticket,
                "expires_in_secs": TICKET_TTL_SECS,
            }))
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `acp`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

// ── SSE ticket ────────────────────────────────────────────────────────────────

/// Issue a short-lived HMAC-signed ticket for browser EventSource SSE auth.
///
/// Ticket format (URL-safe base64 of): `{session_id}:{principal}:{exp}:{hmac_hex}`
///
/// Uses the same `LAB_ACP_HMAC_SECRET` as permission outcome signing.
/// Falls back to a process-ephemeral key if the env var is not set.
fn issue_subscribe_ticket(session_id: &str, principal: &str) -> Result<String, ToolError> {
    let key = load_hmac_key();
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        + TICKET_TTL_SECS;

    let payload = format!("{session_id}:{principal}:{exp}");

    let mut mac = Hmac::<Sha256>::new_from_slice(&key).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("HMAC key error: {e}"),
    })?;
    mac.update(payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_hex = sig
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();

    let ticket = format!("{payload}:{sig_hex}");
    Ok(B64.encode(ticket.as_bytes()))
}

/// Validate an SSE ticket. Returns `(session_id, principal)` on success.
pub fn validate_subscribe_ticket(ticket: &str) -> Result<(String, String), ToolError> {
    let raw = B64.decode(ticket).map_err(|_| ToolError::Sdk {
        sdk_kind: "auth_failed".to_string(),
        message: "invalid ticket encoding".to_string(),
    })?;
    let raw_str = std::str::from_utf8(&raw).map_err(|_| ToolError::Sdk {
        sdk_kind: "auth_failed".to_string(),
        message: "invalid ticket encoding".to_string(),
    })?;

    // Format: session_id:principal:exp:sig_hex
    let parts: Vec<&str> = raw_str.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err(ToolError::Sdk {
            sdk_kind: "auth_failed".to_string(),
            message: "malformed ticket".to_string(),
        });
    }
    let (session_id, principal, exp_str, sig_hex) = (parts[0], parts[1], parts[2], parts[3]);

    let exp: u64 = exp_str.parse().unwrap_or(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now > exp {
        return Err(ToolError::Sdk {
            sdk_kind: "auth_failed".to_string(),
            message: "ticket expired".to_string(),
        });
    }

    let key = load_hmac_key();
    let payload = format!("{session_id}:{principal}:{exp_str}");
    let mut mac = Hmac::<Sha256>::new_from_slice(&key).map_err(|_| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "HMAC key error".to_string(),
    })?;
    mac.update(payload.as_bytes());

    let expected = mac.finalize().into_bytes();
    let expected_hex: String = expected.iter().map(|b| format!("{b:02x}")).collect();
    if expected_hex != sig_hex {
        return Err(ToolError::Sdk {
            sdk_kind: "auth_failed".to_string(),
            message: "invalid ticket signature".to_string(),
        });
    }

    Ok((session_id.to_string(), principal.to_string()))
}

fn load_hmac_key() -> &'static [u8] {
    use std::sync::OnceLock;
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| {
        if let Ok(secret) = std::env::var("LAB_ACP_HMAC_SECRET") {
            if !secret.is_empty() {
                return secret.into_bytes();
            }
        }
        // Fall back to process-ephemeral key — stable for the process lifetime.
        use sha2::Digest;
        let pid = std::process::id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        sha2::Sha256::digest(format!("lab-acp-hmac-ephemeral:{pid}:{now}").as_bytes()).to_vec()
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn help_returns_catalog_object() {
        let v = dispatch("help", json!({})).await.unwrap();
        assert!(v.is_object());
        assert_eq!(v["service"], "acp");
    }

    #[tokio::test]
    async fn schema_returns_action_shape() {
        let v = dispatch("schema", json!({"action": "session.start"}))
            .await
            .unwrap();
        assert!(v.is_object());
    }

    #[tokio::test]
    async fn unknown_action_returns_kind() {
        let e = dispatch("session.serch", json!({})).await.unwrap_err();
        assert_eq!(e.kind(), "unknown_action");
    }

    #[test]
    fn subscribe_ticket_round_trip() {
        let ticket = issue_subscribe_ticket("sess-123", "user@example.com").unwrap();
        let (session_id, principal) = validate_subscribe_ticket(&ticket).unwrap();
        assert_eq!(session_id, "sess-123");
        assert_eq!(principal, "user@example.com");
    }

    #[test]
    fn subscribe_ticket_empty_principal() {
        let ticket = issue_subscribe_ticket("sess-456", "").unwrap();
        let (session_id, principal) = validate_subscribe_ticket(&ticket).unwrap();
        assert_eq!(session_id, "sess-456");
        assert_eq!(principal, "");
    }
}
