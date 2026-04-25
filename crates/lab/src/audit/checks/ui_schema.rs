//! UI schema metadata checks.

use std::path::Path;

use super::super::CheckResult;

pub fn run(name: &str, repo_root: &Path) -> Vec<(String, CheckResult)> {
    let path = repo_root.join(format!("crates/lab-apis/src/{name}.rs"));
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            return vec![(
                "metadata.ui_schema".into(),
                CheckResult::Fail(format!("failed to read {}: {err}", path.display())),
            )];
        }
    };

    vec![
        (
            "metadata.supports_multi_instance".into(),
            supports_multi_instance_check(&content),
        ),
        ("metadata.ui_schema".into(), ui_schema_check(&content)),
        ("metadata.help_url".into(), help_url_check(&content)),
    ]
}

fn supports_multi_instance_check(content: &str) -> CheckResult {
    if content.contains("supports_multi_instance:") {
        CheckResult::Pass
    } else {
        CheckResult::Fail("PluginMeta missing `supports_multi_instance:`".into())
    }
}

fn ui_schema_check(content: &str) -> CheckResult {
    let blocks = env_var_blocks(content);
    if blocks.is_empty() {
        return CheckResult::Pass;
    }

    let missing = blocks
        .iter()
        .enumerate()
        .filter_map(|(idx, block)| (!block.contains("ui: Some(")).then_some(idx + 1))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        CheckResult::Pass
    } else {
        CheckResult::Fail(format!(
            "EnvVar blocks missing explicit `ui: Some(...)`: {}",
            missing
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

fn help_url_check(content: &str) -> CheckResult {
    let invalid = help_urls(content)
        .into_iter()
        .filter(|url| !allowed_help_url(url))
        .collect::<Vec<_>>();

    if invalid.is_empty() {
        CheckResult::Pass
    } else {
        CheckResult::Fail(format!(
            "invalid help_url scheme/host: {}",
            invalid.join(", ")
        ))
    }
}

fn env_var_blocks(content: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut search_from = 0;

    while let Some(relative_start) = content[search_from..].find("EnvVar {") {
        let start = search_from + relative_start;
        let mut depth = 0usize;
        let mut seen_open = false;

        for (offset, ch) in content[start..].char_indices() {
            match ch {
                '{' => {
                    depth += 1;
                    seen_open = true;
                }
                '}' if seen_open => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let end = start + offset + ch.len_utf8();
                        blocks.push(&content[start..end]);
                        search_from = end;
                        break;
                    }
                }
                _ => {}
            }
        }

        if search_from <= start {
            break;
        }
    }

    blocks
}

fn help_urls(content: &str) -> Vec<&str> {
    let mut urls = Vec::new();
    let mut search_from = 0;

    while let Some(relative_start) = content[search_from..].find("help_url: Some(\"") {
        let value_start = search_from + relative_start + "help_url: Some(\"".len();
        let Some(relative_end) = content[value_start..].find('"') else {
            break;
        };
        let value_end = value_start + relative_end;
        urls.push(&content[value_start..value_end]);
        search_from = value_end;
    }

    urls
}

fn allowed_help_url(url: &str) -> bool {
    url.starts_with("https://")
        || localhost_http_url(url, "localhost")
        || localhost_http_url(url, "127.0.0.1")
        || localhost_http_url(url, "[::1]")
}

fn localhost_http_url(url: &str, host: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let Some(after_host) = rest.strip_prefix(host) else {
        return false;
    };
    matches!(
        after_host.as_bytes().first(),
        None | Some(b':' | b'/' | b'?' | b'#')
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_schema_check_passes_when_all_env_vars_have_some_ui() {
        let content = r#"
            pub const META: PluginMeta = PluginMeta {
                supports_multi_instance: false,
                required_env: &[EnvVar { name: "FOO_URL", ui: Some(&URL_FIELD), secret: false, description: "", example: "" }],
                optional_env: &[EnvVar { name: "FOO_TOKEN", ui: Some(&SECRET_OPTIONAL_FIELD), secret: true, description: "", example: "" }],
            };
        "#;
        assert_eq!(ui_schema_check(content), CheckResult::Pass);
    }

    #[test]
    fn ui_schema_check_fails_when_env_var_has_none_or_missing_ui() {
        let content = r#"
            required_env: &[
                EnvVar { name: "FOO_URL", ui: None, secret: false, description: "", example: "" },
                EnvVar { name: "FOO_TOKEN", secret: true, description: "", example: "" },
            ],
        "#;
        assert!(matches!(ui_schema_check(content), CheckResult::Fail(_)));
    }

    #[test]
    fn supports_multi_instance_check_requires_field() {
        assert_eq!(
            supports_multi_instance_check("supports_multi_instance: false"),
            CheckResult::Pass
        );
        assert!(matches!(
            supports_multi_instance_check("pub const META: PluginMeta = PluginMeta {};"),
            CheckResult::Fail(_)
        ));
    }

    #[test]
    fn help_url_check_allows_https_and_localhost_http() {
        let content = r#"
            UiSchema { help_url: Some("https://example.com"), ..UI_SCHEMA_DEFAULT };
            UiSchema { help_url: Some("http://localhost:3000/help"), ..UI_SCHEMA_DEFAULT };
            UiSchema { help_url: Some("http://127.0.0.1/help"), ..UI_SCHEMA_DEFAULT };
            UiSchema { help_url: Some("http://[::1]/help"), ..UI_SCHEMA_DEFAULT };
        "#;
        assert_eq!(help_url_check(content), CheckResult::Pass);
    }

    #[test]
    fn help_url_check_rejects_public_http() {
        let content = r#"UiSchema { help_url: Some("http://example.com"), ..UI_SCHEMA_DEFAULT };"#;
        assert!(matches!(help_url_check(content), CheckResult::Fail(_)));
    }
}
