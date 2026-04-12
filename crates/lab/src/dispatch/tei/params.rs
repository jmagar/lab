use lab_apis::tei::types::EmbedRequest;
use serde_json::Value;

use crate::dispatch::error::ToolError;

pub fn embed_request_from_params(params: &Value) -> Result<EmbedRequest, ToolError> {
    let source = params
        .get("payload")
        .cloned()
        .unwrap_or_else(|| params.clone());

    let mut input_strings = match source.get("inputs") {
        Some(Value::String(value)) => vec![value.clone()],
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| ToolError::InvalidParam {
                        message: "parameter `inputs` must be an array of strings".into(),
                        param: "inputs".into(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => {
            return Err(ToolError::InvalidParam {
                message: "parameter `inputs` must be a string or array of strings".into(),
                param: "inputs".into(),
            });
        }
        None => Vec::new(),
    };

    if input_strings.is_empty() {
        if let Some(value) = source.get("input").and_then(Value::as_str) {
            input_strings.push(value.to_owned());
        }
    }

    if input_strings.is_empty() {
        return Err(ToolError::MissingParam {
            message: "missing required parameter `inputs`".into(),
            param: "inputs".into(),
        });
    }

    Ok(EmbedRequest {
        inputs: input_strings,
        normalize: optional_bool(&source, "normalize")?,
        truncate: optional_bool(&source, "truncate")?,
    })
}

fn optional_bool(source: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    match source.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a boolean"),
                param: key.into(),
            }),
    }
}
