//! Structured JSON envelopes returned by every MCP tool dispatch.
//! Shape is identical to what the HTTP API emits (see `api/error.rs`)
//! so clients can share error-handling logic across transports.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// Successful tool result wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolEnvelope<T> {
    /// The tool's result payload.
    pub data: T,
}

impl<T> ToolEnvelope<T> {
    /// Wrap a value in an envelope.
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

/// Error variants that MCP dispatchers can produce on top of SDK errors.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolError {
    /// Action name not recognized for this service.
    UnknownAction {
        /// Human-readable message.
        message: String,
        /// Valid action names for this service.
        valid: Vec<String>,
        /// Optional fuzzy suggestion.
        hint: Option<String>,
    },
    /// Required parameter missing.
    MissingParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Parameter present but wrong type or value.
    InvalidParam {
        /// Human-readable message.
        message: String,
        /// Parameter name.
        param: String,
    },
    /// Multi-instance label not found.
    UnknownInstance {
        /// Human-readable message.
        message: String,
        /// Known instance labels.
        valid: Vec<String>,
    },
    /// Pass-through of an `ApiError::kind()` tag from the SDK.
    Sdk {
        /// Stable kind tag (`auth_failed`, `rate_limited`, …).
        #[serde(rename = "sdk_kind")]
        sdk_kind: String,
        /// Human-readable message.
        message: String,
    },
}

impl Serialize for ToolError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::UnknownAction {
                message,
                valid,
                hint,
            } => serde_json::json!({
                "kind": "unknown_action",
                "message": message,
                "valid": valid,
                "hint": hint,
            }),
            Self::MissingParam { message, param } => serde_json::json!({
                "kind": "missing_param",
                "message": message,
                "param": param,
            }),
            Self::InvalidParam { message, param } => serde_json::json!({
                "kind": "invalid_param",
                "message": message,
                "param": param,
            }),
            Self::UnknownInstance { message, valid } => serde_json::json!({
                "kind": "unknown_instance",
                "message": message,
                "valid": valid,
            }),
            Self::Sdk { sdk_kind, message } => serde_json::json!({
                "kind": sdk_kind,
                "message": message,
            }),
        };
        v.serialize(serializer)
    }
}

impl IntoResponse for ToolError {
    fn into_response(self) -> Response {
        let status = match self.kind() {
            "auth_failed" => StatusCode::UNAUTHORIZED,
            "not_found" => StatusCode::NOT_FOUND,
            "rate_limited" => StatusCode::TOO_MANY_REQUESTS,
            "missing_param" | "invalid_param" | "validation_failed" => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            "unknown_action" | "unknown_subaction" | "unknown_instance" => StatusCode::BAD_REQUEST,
            "network_error" | "server_error" => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        // Serialize self directly — byte-identical to the MCP error envelope.
        let body = serde_json::to_value(&self).unwrap_or_else(|_| {
            serde_json::json!({"kind": "internal_error", "message": "error serialization failed"})
        });
        (status, axum::Json(body)).into_response()
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Serialize to the stable JSON envelope so callers get a machine-readable string.
        match serde_json::to_string(self) {
            Ok(s) => f.write_str(&s),
            Err(_) => write!(f, "{self:?}"),
        }
    }
}

impl ToolError {
    /// Canonical stable string tag.
    #[must_use]
    pub const fn kind(&self) -> &str {
        match self {
            Self::UnknownAction { .. } => "unknown_action",
            Self::MissingParam { .. } => "missing_param",
            Self::InvalidParam { .. } => "invalid_param",
            Self::UnknownInstance { .. } => "unknown_instance",
            Self::Sdk { sdk_kind, .. } => sdk_kind.as_str(),
        }
    }
}

// ── Spec-conformant envelope builders ────────────────────────────────────────
//
// `serve.rs` uses these to produce the wire shape required by the MCP spec:
//   success: `{ ok: true,  service, action, data }`
//   error:   `{ ok: false, service, action, error: { kind, message, … } }`

use serde_json::{Value, json};

/// Build a success envelope.
///
/// ```json
/// { "ok": true, "service": "radarr", "action": "movie.list", "data": […] }
/// ```
#[must_use]
pub fn build_success(service: &str, action: &str, data: &Value) -> Value {
    json!({
        "ok": true,
        "service": service,
        "action": action,
        "data": data,
    })
}

/// Build an error envelope.
///
/// ```json
/// { "ok": false, "service": "radarr", "action": "movie.add",
///   "error": { "kind": "missing_param", "message": "…" } }
/// ```
#[must_use]
pub fn build_error(service: &str, action: &str, kind: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "service": service,
        "action": action,
        "error": {
            "kind": kind,
            "message": message,
        },
    })
}

/// Build an error envelope with extra structured fields (e.g. `valid`, `param`).
#[must_use]
pub fn build_error_extra(
    service: &str,
    action: &str,
    kind: &str,
    message: &str,
    extra: &Value,
) -> Value {
    let mut obj = build_error(service, action, kind, message);
    if let (Some(err), Some(ext_map)) = (
        obj.get_mut("error").and_then(Value::as_object_mut),
        extra.as_object(),
    ) {
        for (k, v) in ext_map {
            err.insert(k.clone(), v.clone());
        }
    }
    obj
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── kind() tags ──────────────────────────────────────────────────────────

    #[test]
    fn kind_unknown_action() {
        let e = ToolError::UnknownAction {
            message: "bad".into(),
            valid: vec![],
            hint: None,
        };
        assert_eq!(e.kind(), "unknown_action");
    }

    #[test]
    fn kind_missing_param() {
        let e = ToolError::MissingParam {
            message: "x".into(),
            param: "id".into(),
        };
        assert_eq!(e.kind(), "missing_param");
    }

    #[test]
    fn kind_invalid_param() {
        let e = ToolError::InvalidParam {
            message: "x".into(),
            param: "id".into(),
        };
        assert_eq!(e.kind(), "invalid_param");
    }

    #[test]
    fn kind_unknown_instance() {
        let e = ToolError::UnknownInstance {
            message: "x".into(),
            valid: vec![],
        };
        assert_eq!(e.kind(), "unknown_instance");
    }

    #[test]
    fn kind_sdk_uses_sdk_kind_value() {
        let e = ToolError::Sdk {
            sdk_kind: "auth_failed".into(),
            message: "denied".into(),
        };
        assert_eq!(e.kind(), "auth_failed");
    }

    // ── JSON shape (kind field is the semantic tag, not the variant name) ────

    fn json(e: &ToolError) -> Value {
        serde_json::to_value(e).expect("ToolError is always serializable")
    }

    #[test]
    fn json_unknown_action_shape() {
        let e = ToolError::UnknownAction {
            message: "oops".into(),
            valid: vec!["a".into(), "b".into()],
            hint: Some("did you mean a?".into()),
        };
        let v = json(&e);
        assert_eq!(v["kind"], "unknown_action");
        assert_eq!(v["message"], "oops");
        assert_eq!(v["valid"], serde_json::json!(["a", "b"]));
        assert_eq!(v["hint"], "did you mean a?");
    }

    #[test]
    fn json_missing_param_shape() {
        let e = ToolError::MissingParam {
            message: "need id".into(),
            param: "id".into(),
        };
        let v = json(&e);
        assert_eq!(v["kind"], "missing_param");
        assert_eq!(v["param"], "id");
    }

    #[test]
    fn json_sdk_promotes_sdk_kind_to_kind() {
        // The Sdk variant must NOT serialize as {"kind":"sdk",...}.
        // It must promote the sdk_kind value to the kind field.
        let e = ToolError::Sdk {
            sdk_kind: "auth_failed".into(),
            message: "denied".into(),
        };
        let v = json(&e);
        assert_eq!(
            v["kind"], "auth_failed",
            "sdk_kind must be promoted to kind field"
        );
        assert!(
            v.get("sdk_kind").is_none(),
            "sdk_kind must not appear as a separate field"
        );
        assert_eq!(v["message"], "denied");
    }

    #[test]
    fn json_sdk_not_found() {
        let e = ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: "missing".into(),
        };
        let v = json(&e);
        assert_eq!(v["kind"], "not_found");
    }

    // ── Display produces valid parseable JSON ────────────────────────────────

    #[test]
    fn display_is_valid_json() {
        let e = ToolError::MissingParam {
            message: "need it".into(),
            param: "q".into(),
        };
        let s = e.to_string();
        let parsed: Value = serde_json::from_str(&s).expect("Display output must be valid JSON");
        assert_eq!(parsed["kind"], "missing_param");
    }

    #[test]
    fn display_sdk_has_correct_kind() {
        let e = ToolError::Sdk {
            sdk_kind: "rate_limited".into(),
            message: "slow down".into(),
        };
        let s = e.to_string();
        let parsed: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(parsed["kind"], "rate_limited");
    }

    // ── build_success / build_error wire shape ───────────────────────────────

    #[test]
    fn success_envelope_shape() {
        let env = build_success("radarr", "movie.list", &json!([{"id": 1}]));
        assert_eq!(env["ok"], json!(true));
        assert_eq!(env["service"], json!("radarr"));
        assert_eq!(env["action"], json!("movie.list"));
        assert!(env["data"].is_array());
        assert!(env.get("error").is_none());
    }

    #[test]
    fn error_envelope_shape() {
        let env = build_error("radarr", "movie.add", "missing_param", "missing `title`");
        assert_eq!(env["ok"], json!(false));
        assert_eq!(env["service"], json!("radarr"));
        assert_eq!(env["action"], json!("movie.add"));
        assert_eq!(env["error"]["kind"], json!("missing_param"));
        assert!(env["error"]["message"].as_str().is_some());
        assert!(env.get("data").is_none());
    }

    #[test]
    fn error_extra_merges_valid_list() {
        let env = build_error_extra(
            "radarr",
            "bad.action",
            "unknown_action",
            "unknown action",
            &json!({ "valid": ["movie.list"], "param": null, "hint": null }),
        );
        assert_eq!(env["error"]["kind"], json!("unknown_action"));
        assert!(env["error"]["valid"].is_array());
    }

    #[test]
    fn success_has_no_error_key() {
        let env = build_success("extract", "scan", &json!({}));
        let s = serde_json::to_string(&env).unwrap();
        assert!(!s.contains("\"error\""));
    }

    #[test]
    fn error_has_no_data_key() {
        let env = build_error("extract", "scan", "network_error", "refused");
        let s = serde_json::to_string(&env).unwrap();
        assert!(!s.contains("\"data\""));
    }
}
