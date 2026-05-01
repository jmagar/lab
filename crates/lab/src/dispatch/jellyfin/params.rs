use lab_apis::jellyfin::types::ItemsQuery;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{optional_str, optional_u32, require_str};

pub fn item_id(params: &Value) -> Result<&str, ToolError> {
    require_str(params, "item_id")
}

pub fn items_query(params: &Value) -> Result<ItemsQuery, ToolError> {
    Ok(ItemsQuery {
        user_id: optional_string(params, "user_id")?,
        search_term: optional_string(params, "search_term")?,
        parent_id: optional_string(params, "parent_id")?,
        include_item_types: optional_string_list(params, "include_item_types")?,
        recursive: optional_bool(params, "recursive")?,
        start_index: optional_u32(params, "start_index")?,
        limit: optional_u32(params, "limit")?,
    })
}

fn optional_string(params: &Value, key: &str) -> Result<Option<String>, ToolError> {
    optional_str(params, key).map(|value| value.map(str::to_string))
}

fn optional_bool(params: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    params.get(key).map_or(Ok(None), |value| {
        value
            .as_bool()
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a boolean"),
                param: key.to_string(),
            })
    })
}

fn optional_string_list(params: &Value, key: &str) -> Result<Option<Vec<String>>, ToolError> {
    let Some(value) = params.get(key) else {
        return Ok(None);
    };
    let values = value.as_array().ok_or_else(|| ToolError::InvalidParam {
        message: format!("parameter `{key}` must be an array of strings"),
        param: key.to_string(),
    })?;

    let mut out = Vec::with_capacity(values.len());
    for item in values {
        let value = item.as_str().ok_or_else(|| ToolError::InvalidParam {
            message: format!("parameter `{key}` must be an array of strings"),
            param: key.to_string(),
        })?;
        if value.is_empty() {
            return Err(ToolError::InvalidParam {
                message: format!("parameter `{key}` must not contain empty strings"),
                param: key.to_string(),
            });
        }
        out.push(value.to_string());
    }
    Ok(Some(out))
}
