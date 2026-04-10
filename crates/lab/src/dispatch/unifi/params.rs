use serde_json::{Map, Value};

use crate::dispatch::error::ToolError;

pub use crate::dispatch::helpers::{require_i64, require_str, to_json};

#[allow(dead_code)]
pub fn require_u32(params: &Value, key: &str) -> Result<u32, ToolError> {
    let v = require_i64(params, key)?;
    u32::try_from(v).map_err(|_| ToolError::InvalidParam {
        message: format!("parameter `{key}` must be a non-negative integer"),
        param: key.to_string(),
    })
}

pub fn object_without(params: &Value, excluded: &[&str]) -> Result<Value, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let filtered: Map<String, Value> = obj
        .iter()
        .filter(|(k, _)| !excluded.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    Ok(Value::Object(filtered))
}

pub fn query_from(params: &Value, keys: &[&str]) -> Result<Vec<(String, String)>, ToolError> {
    let obj = params.as_object().ok_or_else(|| ToolError::InvalidParam {
        message: "expected JSON object".to_string(),
        param: "params".to_string(),
    })?;
    let mut out = Vec::new();
    for key in keys {
        if let Some(v) = obj.get(*key) {
            let rendered = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            out.push(((*key).to_string(), rendered));
        }
    }
    Ok(out)
}
