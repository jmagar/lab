use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookLmAuth {
    pub cookies: Vec<NotebookLmCookie>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookLmCookie {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_unix: Option<i64>,
    pub is_owner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub(crate) const LIST_NOTEBOOKS: &str = "wXbhsf";
pub(crate) const CREATE_NOTEBOOK: &str = "CCqFvf";
pub(crate) const GET_NOTEBOOK: &str = "rLM1Ne";
pub(crate) const DELETE_NOTEBOOK: &str = "WWINqb";
pub(crate) const ADD_SOURCE: &str = "izAoDd";

pub(crate) fn notebook_from_api(data: &Value) -> Notebook {
    let arr = data.as_array().map(Vec::as_slice).unwrap_or(&[]);
    let title = arr
        .first()
        .and_then(Value::as_str)
        .unwrap_or_default()
        .replace("thought\n", "")
        .trim()
        .to_string();
    let id = arr
        .get(2)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let created_at_unix = arr
        .get(5)
        .and_then(Value::as_array)
        .and_then(|v| v.get(5))
        .and_then(Value::as_array)
        .and_then(|v| v.first())
        .and_then(Value::as_i64);
    let is_owner = arr
        .get(5)
        .and_then(Value::as_array)
        .and_then(|v| v.get(1))
        .and_then(Value::as_bool)
        .is_none_or(|shared| !shared);

    Notebook {
        id,
        title,
        created_at_unix,
        is_owner,
    }
}

pub(crate) fn sources_from_notebook_response(data: &Value) -> Vec<Source> {
    let Some(sources) = data
        .as_array()
        .and_then(|outer| outer.first())
        .and_then(Value::as_array)
        .and_then(|nb| nb.get(1))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    sources
        .iter()
        .filter_map(|source| source_from_list_entry(source, false))
        .collect()
}

pub(crate) fn source_from_api(data: &Value) -> Option<Source> {
    source_from_nested_add_response(data)
}

fn source_from_nested_add_response(data: &Value) -> Option<Source> {
    if let Some(source) = source_from_list_entry(data, true) {
        return Some(source);
    }
    for child in data.as_array()? {
        if let Some(source) = source_from_nested_add_response(child) {
            return Some(source);
        }
    }
    None
}

fn source_from_list_entry(data: &Value, allow_metadata_zero_url: bool) -> Option<Source> {
    let arr = data.as_array()?;
    let id = arr
        .first()
        .and_then(|id| {
            id.as_str()
                .or_else(|| id.as_array()?.first()?.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_default();
    if id.is_empty() {
        return None;
    }

    let metadata = arr.get(2).and_then(Value::as_array);
    let url = metadata.and_then(|m| extract_source_url(m, allow_metadata_zero_url));
    let kind = metadata
        .and_then(|m| m.get(4))
        .and_then(Value::as_i64)
        .map(source_kind);
    let status = arr
        .get(3)
        .and_then(Value::as_array)
        .and_then(|v| v.get(1))
        .and_then(Value::as_i64)
        .map(source_status);

    Some(Source {
        id,
        title: arr.get(1).and_then(Value::as_str).map(ToString::to_string),
        url,
        kind,
        status,
    })
}

fn extract_source_url(metadata: &[Value], allow_zero_fallback: bool) -> Option<String> {
    let indexes: &[usize] = if allow_zero_fallback {
        &[7, 5, 0]
    } else {
        &[7, 5]
    };
    for idx in indexes {
        if let Some(url) = metadata.get(*idx).and_then(Value::as_str)
            && url.starts_with("http")
        {
            return Some(url.to_string());
        }
        if let Some(url) = metadata
            .get(*idx)
            .and_then(Value::as_array)
            .and_then(|v| v.first())
            .and_then(Value::as_str)
            && url.starts_with("http")
        {
            return Some(url.to_string());
        }
    }
    None
}

fn source_kind(code: i64) -> String {
    match code {
        1 => "google_docs",
        2 => "google_slides",
        3 => "pdf",
        4 => "pasted_text",
        5 => "web_page",
        8 => "markdown",
        9 => "youtube",
        10 => "media",
        11 => "docx",
        13 => "image",
        14 => "google_spreadsheet",
        16 => "csv",
        17 => "epub",
        _ => "unknown",
    }
    .to_string()
}

fn source_status(code: i64) -> String {
    match code {
        1 => "processing",
        2 => "ready",
        3 => "error",
        4 => "preparing",
        _ => "unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_notebook_shape_from_notebooklm_py_contract() {
        let raw = serde_json::json!([
            "thought\nResearch",
            null,
            "nb-123",
            null,
            null,
            [null, false, null, null, null, [1_706_000_000]]
        ]);
        let notebook = notebook_from_api(&raw);
        assert_eq!(notebook.id, "nb-123");
        assert_eq!(notebook.title, "Research");
        assert!(notebook.is_owner);
        assert_eq!(notebook.created_at_unix, Some(1_706_000_000));
    }

    #[test]
    fn parses_sources_from_get_notebook_payload() {
        let raw = serde_json::json!([[
            null,
            [[
                ["src-1"],
                "Example",
                [null, null, null, null, 5, "https://example.com"],
                [null, 2]
            ]]
        ]]);
        let sources = sources_from_notebook_response(&raw);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].id, "src-1");
        assert_eq!(sources[0].kind.as_deref(), Some("web_page"));
        assert_eq!(sources[0].status.as_deref(), Some("ready"));
    }

    #[test]
    fn add_source_preserves_nested_metadata() {
        let raw = serde_json::json!([[[[
            ["src-1"],
            "Example",
            [
                "https://fallback.example",
                null,
                null,
                null,
                5,
                "https://example.com"
            ],
            [null, 2]
        ]]]]);
        let source = source_from_api(&raw).unwrap();
        assert_eq!(source.id, "src-1");
        assert_eq!(source.title.as_deref(), Some("Example"));
        assert_eq!(source.url.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn list_source_url_does_not_use_metadata_zero_fallback() {
        let raw = serde_json::json!([[
            null,
            [[
                ["src-1"],
                "Example",
                ["https://wrong.example", null, null, null, 5],
                [null, 2]
            ]]
        ]]);
        let sources = sources_from_notebook_response(&raw);
        assert_eq!(sources[0].url, None);
    }
}
