use lab_apis::neo4j::types::{Statement, TxMode};
use serde_json::{Map, Value};

use crate::dispatch::error::ToolError;

pub fn optional_object(params: &Value, key: &str) -> Result<Map<String, Value>, ToolError> {
    match params.get(key) {
        None => Ok(Map::new()),
        Some(Value::Object(map)) => Ok(map.clone()),
        Some(_) => Err(ToolError::InvalidParam {
            message: format!("parameter `{key}` must be an object"),
            param: key.to_string(),
        }),
    }
}

pub fn statements(params: &Value) -> Result<Vec<Statement>, ToolError> {
    let Some(Value::Array(values)) = params.get("statements") else {
        return Err(ToolError::MissingParam {
            message: "missing required parameter `statements`".into(),
            param: "statements".into(),
        });
    };
    values
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let Value::Object(map) = value else {
                return Err(ToolError::InvalidParam {
                    message: format!("statement #{idx} must be an object"),
                    param: "statements".into(),
                });
            };
            let statement = map
                .get("statement")
                .and_then(Value::as_str)
                .ok_or_else(|| ToolError::MissingParam {
                    message: format!("missing `statement` in statements[{idx}]"),
                    param: "statement".into(),
                })?;
            let parameters = match map.get("parameters") {
                None => Map::new(),
                Some(Value::Object(parameters)) => parameters.clone(),
                Some(_) => {
                    return Err(ToolError::InvalidParam {
                        message: format!("parameters in statements[{idx}] must be an object"),
                        param: "parameters".into(),
                    });
                }
            };
            Ok(Statement {
                statement: statement.to_string(),
                parameters,
            })
        })
        .collect()
}

pub fn tx_mode(params: &Value) -> Result<TxMode, ToolError> {
    match params.get("mode").and_then(Value::as_str) {
        Some("r" | "read") => Ok(TxMode::R),
        Some("w" | "write") => Ok(TxMode::W),
        Some(_) => Err(ToolError::InvalidParam {
            message: "parameter `mode` must be `r` or `w`".into(),
            param: "mode".into(),
        }),
        None => Err(ToolError::MissingParam {
            message: "missing required parameter `mode`".into(),
            param: "mode".into(),
        }),
    }
}
