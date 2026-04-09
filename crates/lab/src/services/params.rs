//! Shared key=value param parsing for action-style CLI subcommands.
//!
//! Both ByteStash and UniFi (and future services) expose an `action + params`
//! dispatch surface. This module owns the canonical parse/coerce logic so it
//! is not duplicated per service.

use anyhow::Result;
use serde_json::{Map, Value};

/// Parse a list of `key=value` strings into a JSON object.
///
/// Each string must contain exactly one `=`. The value portion is coerced to
/// a JSON boolean, integer, float, or string — in that precedence order.
///
/// # Errors
/// Returns an error if any item does not contain `=`.
pub fn parse_kv_params(params: Vec<String>) -> Result<Value> {
    let mut map = Map::new();
    for item in params {
        let Some((key, raw)) = item.split_once('=') else {
            anyhow::bail!("invalid param `{item}`; expected key=value");
        };
        map.insert(key.to_string(), coerce_value(raw));
    }
    Ok(Value::Object(map))
}

/// Coerce a raw string into the most specific JSON scalar type.
fn coerce_value(raw: &str) -> Value {
    if raw.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if raw.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(n) = raw.parse::<f64>()
        && let Some(num) = serde_json::Number::from_f64(n)
    {
        return Value::Number(num);
    }
    Value::String(raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kv_params_coerces_scalars() {
        let value = parse_kv_params(vec![
            "enabled=true".to_string(),
            "count=7".to_string(),
            "ratio=1.5".to_string(),
            "name=alice".to_string(),
        ])
        .unwrap();
        assert_eq!(value["enabled"], true);
        assert_eq!(value["count"], 7);
        assert_eq!(value["ratio"], 1.5);
        assert_eq!(value["name"], "alice");
    }

    #[test]
    fn parse_kv_params_rejects_missing_equals() {
        let err = parse_kv_params(vec!["broken".to_string()]).unwrap_err();
        assert!(err.to_string().contains("expected key=value"));
    }

    #[test]
    fn parse_kv_params_empty_returns_empty_object() {
        let value = parse_kv_params(vec![]).unwrap();
        assert!(value.as_object().unwrap().is_empty());
    }
}
