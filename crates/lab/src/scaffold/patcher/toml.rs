//! TOML patchers for scaffold generation.

use super::super::service::{Result, ScaffoldError};

pub fn patch_lab_apis_cargo(name: &str, content: &str) -> Result<String> {
    patch_feature_section(content, name, |_| toml::Value::Array(vec![]))
}

pub fn patch_lab_cargo(name: &str, content: &str) -> Result<String> {
    patch_feature_section(content, name, |service| {
        toml::Value::Array(vec![toml::Value::String(format!("lab-apis/{service}"))])
    })
}

fn patch_feature_section<F>(content: &str, name: &str, render: F) -> Result<String>
where
    F: FnOnce(&str) -> toml::Value,
{
    let Some((prefix, body, suffix)) = split_feature_section(content) else {
        return Err(ScaffoldError::Toml {
            message: "missing [features] section".into(),
        });
    };

    let mut doc: toml::Table = toml::from_str(&format!("[features]\n{body}")).map_err(
        |err: toml::de::Error| ScaffoldError::Toml {
            message: err.to_string(),
        },
    )?;
    let Some(features): Option<&mut toml::map::Map<String, toml::Value>> =
        doc.get_mut("features").and_then(toml::Value::as_table_mut)
    else {
        return Err(ScaffoldError::Toml {
            message: "missing [features] table".into(),
        });
    };

    if features.contains_key(name) {
        return Ok(content.to_string());
    }

    let mut reordered: toml::map::Map<String, toml::Value> = toml::map::Map::new();
    let rendered = render(name);
    let mut inserted = false;
    for (key, value) in features.iter() {
        if !inserted && key == "all" {
            reordered.insert(name.to_string(), rendered.clone());
            inserted = true;
        }
        reordered.insert(key.clone(), value.clone());
    }
    if !inserted {
        reordered.insert(name.to_string(), rendered);
    }
    *features = reordered;

    let rendered = toml::to_string_pretty(&doc)
        .map_err(|err| ScaffoldError::Toml {
            message: err.to_string(),
        })?;
    let rendered_body = rendered
        .strip_prefix("[features]\n")
        .or_else(|| rendered.strip_prefix("[features]\r\n"))
        .unwrap_or(rendered.as_str());

    let mut out = String::with_capacity(prefix.len() + rendered_body.len() + suffix.len());
    out.push_str(prefix);
    out.push_str(rendered_body);
    out.push_str(suffix);
    Ok(out)
}

fn split_feature_section(content: &str) -> Option<(&str, &str, &str)> {
    let mut prefix_end = None;
    let mut feature_start = None;
    let mut feature_end = None;
    let mut offset = 0usize;

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if feature_start.is_none() && trimmed == "[features]" {
            feature_start = Some(offset);
            prefix_end = Some(offset + line.len());
        } else if feature_start.is_some()
            && trimmed.starts_with('[')
            && trimmed.ends_with(']')
            && trimmed != "[features]"
        {
            feature_end = Some(offset);
            break;
        }
        offset += line.len();
    }

    feature_start?;
    let prefix_end = prefix_end?;
    let end = feature_end.unwrap_or(content.len());
    let prefix = &content[..prefix_end];
    let body = &content[prefix_end..end];
    let suffix = &content[end..];
    Some((prefix, body, suffix))
}
