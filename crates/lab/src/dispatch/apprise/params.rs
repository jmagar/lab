use lab_apis::apprise::types::{BodyFormat, NotifyRequest, NotifyType};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::object_without;

pub fn notify_request_from_params(
    params: &Value,
    strip: &[&str],
) -> Result<NotifyRequest, ToolError> {
    let source = match payload_object(params)? {
        Some(payload) => payload,
        None => object_without(params, strip),
    };

    let body = source
        .get("body")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: "missing required parameter `body`".into(),
            param: "body".into(),
        })?
        .to_owned();

    Ok(NotifyRequest {
        body,
        title: optional_string(&source, "title")?,
        urls: optional_string_list(&source, "urls")?,
        notify_type: optional_notify_type(&source)?,
        format: optional_body_format(&source)?,
        tag: optional_string(&source, "tag")?,
    })
}

fn payload_object(params: &Value) -> Result<Option<Value>, ToolError> {
    match params.get("payload") {
        None => Ok(None),
        Some(Value::Object(map)) => Ok(Some(Value::Object(map.clone()))),
        Some(Value::String(raw)) => {
            let parsed = serde_json::from_str::<Value>(raw).map_err(|_| ToolError::InvalidParam {
                message: "parameter `payload` must be a JSON object".into(),
                param: "payload".into(),
            })?;
            if !parsed.is_object() {
                return Err(ToolError::InvalidParam {
                    message: "parameter `payload` must be a JSON object".into(),
                    param: "payload".into(),
                });
            }
            Ok(Some(parsed))
        }
        Some(_) => Err(ToolError::InvalidParam {
            message: "parameter `payload` must be a JSON object".into(),
            param: "payload".into(),
        }),
    }
}

fn optional_string(source: &Value, key: &str) -> Result<Option<String>, ToolError> {
    match source.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ToolError::InvalidParam {
            message: format!("parameter `{key}` must be a string"),
            param: key.into(),
        }),
    }
}

fn optional_string_list(source: &Value, key: &str) -> Result<Option<Vec<String>>, ToolError> {
    match source.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(vec![value.clone()])),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| ToolError::InvalidParam {
                        message: format!("parameter `{key}` must be an array of strings"),
                        param: key.into(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(ToolError::InvalidParam {
            message: format!("parameter `{key}` must be a string or array of strings"),
            param: key.into(),
        }),
    }
}

fn optional_notify_type(source: &Value) -> Result<Option<NotifyType>, ToolError> {
    match source.get("type") {
        None => Ok(None),
        Some(Value::String(value)) => match value.as_str() {
            "info" => Ok(Some(NotifyType::Info)),
            "success" => Ok(Some(NotifyType::Success)),
            "warning" => Ok(Some(NotifyType::Warning)),
            "failure" => Ok(Some(NotifyType::Failure)),
            _ => Err(ToolError::InvalidParam {
                message: "parameter `type` must be one of: info, success, warning, failure".into(),
                param: "type".into(),
            }),
        },
        Some(_) => Err(ToolError::InvalidParam {
            message: "parameter `type` must be a string".into(),
            param: "type".into(),
        }),
    }
}

fn optional_body_format(source: &Value) -> Result<Option<BodyFormat>, ToolError> {
    match source.get("format") {
        None => Ok(None),
        Some(Value::String(value)) => match value.as_str() {
            "text" => Ok(Some(BodyFormat::Text)),
            "markdown" => Ok(Some(BodyFormat::Markdown)),
            "html" => Ok(Some(BodyFormat::Html)),
            _ => Err(ToolError::InvalidParam {
                message: "parameter `format` must be one of: text, markdown, html".into(),
                param: "format".into(),
            }),
        },
        Some(_) => Err(ToolError::InvalidParam {
            message: "parameter `format` must be a string".into(),
            param: "format".into(),
        }),
    }
}
