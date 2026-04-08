//! Structured JSON envelopes returned by every MCP transport layer.
//!
//! Success shape  : `{ ok: true,  service, action, data }`
//! Error shape    : `{ ok: false, service, action, error: { kind, message, … } }`
//!
//! Both shapes are built by the `serve` layer, not by individual dispatchers.

use serde_json::{Value, json};

/// Build a success envelope.
///
/// ```json
/// { "ok": true, "service": "radarr", "action": "movie.list", "data": […] }
/// ```
#[must_use]
pub fn build_success(service: &str, action: &str, data: Value) -> Value {
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
    extra: Value,
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
