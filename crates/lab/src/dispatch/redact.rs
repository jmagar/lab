use reqwest::Url;

pub fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    matches!(
        normalized.as_str(),
        "token"
            | "access_token"
            | "id_token"
            | "refresh_token"
            | "apikey"
            | "api_key"
            | "password"
            | "passwd"
            | "secret"
            | "client_secret"
            | "authorization"
            | "bearer"
            | "session"
            | "session_id"
            | "cookie"
            | "code"
    ) || normalized.ends_with("_token")
        || normalized.ends_with("_secret")
        || normalized.ends_with("_password")
        || normalized.ends_with("_key")
}

pub fn redact_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(parsed) => redact_parsed_url(parsed),
        Err(_) => "[invalid-url-redacted]".to_string(),
    }
}

pub fn redact_stdio_value(value: &str) -> String {
    if let Some((key, _)) = value.split_once('=')
        && is_sensitive_key(key)
    {
        return format!("{key}=[redacted]");
    }

    if let Some(flag) = value.strip_prefix("--") {
        let (key, _) = flag.split_once('=').map_or((flag, ""), |(k, v)| (k, v));
        if is_sensitive_key(key) {
            return format!("--{key}=[redacted]");
        }
    }

    value.to_string()
}

pub fn redact_stdio_args(args: &[String]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut redact_next = false;

    for arg in args {
        if redact_next {
            redacted.push("[redacted]".to_string());
            redact_next = false;
            continue;
        }

        let is_sensitive_flag = arg
            .strip_prefix("--")
            .map(|value| value.split_once('=').map_or(value, |(key, _)| key))
            .is_some_and(is_sensitive_key);

        if is_sensitive_flag && !arg.contains('=') {
            redacted.push(arg.clone());
            redact_next = true;
            continue;
        }

        redacted.push(redact_stdio_value(arg));
    }

    redacted
}

pub fn redact_upstream_resource_uri(uri: &str) -> String {
    let Some(rest) = uri.strip_prefix("lab://upstream/") else {
        return redact_url(uri);
    };
    let Some(slash_pos) = rest.find('/') else {
        return "lab://upstream/[redacted]".to_string();
    };
    let upstream_name = &rest[..slash_pos];
    let original_uri = &rest[slash_pos + 1..];
    // Preserve non-sensitive pagination/id query params so observability can
    // still distinguish resources; only `is_sensitive_key` entries are masked.
    let redacted_original = redact_uri_or_path(original_uri);
    format!("lab://upstream/{upstream_name}/{redacted_original}")
}

fn redact_uri_or_path(value: &str) -> String {
    if let Ok(parsed) = Url::parse(value) {
        return redact_parsed_url(parsed);
    }
    let (path, query) = match value.split_once('?') {
        Some((path, rest)) => (path.split('#').next().unwrap_or(path), Some(rest)),
        None => (value.split('#').next().unwrap_or(value), None),
    };
    match query.map(redact_query_pairs) {
        Some(q) if !q.is_empty() => format!("{path}?{q}"),
        _ => path.to_string(),
    }
}

fn redact_parsed_url(mut parsed: Url) -> String {
    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);
    let redacted_query = parsed.query().map(redact_query_pairs);
    parsed.set_query(redacted_query.as_deref());
    parsed.set_fragment(None);
    parsed.to_string()
}

fn redact_query_pairs(query: &str) -> String {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').map_or((pair, ""), |(k, v)| (k, v));
            if is_sensitive_key(key) {
                format!("{key}=[redacted]")
            } else if value.is_empty() {
                key.to_string()
            } else {
                format!("{key}={value}")
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_stdio_args_masks_split_form_secret_flags() {
        let args = vec![
            "npx".to_string(),
            "--api-key".to_string(),
            "super-secret".to_string(),
            "--token=abc123".to_string(),
        ];

        assert_eq!(
            redact_stdio_args(&args),
            vec![
                "npx".to_string(),
                "--api-key".to_string(),
                "[redacted]".to_string(),
                "--token=[redacted]".to_string(),
            ]
        );
    }

    #[test]
    fn redact_url_masks_credentials_and_sensitive_query_values() {
        assert_eq!(
            redact_url("http://user:pass@example.com/callback?token=secret&mode=1"),
            "http://example.com/callback?token=[redacted]&mode=1"
        );
    }

    #[test]
    fn redact_upstream_resource_uri_masks_embedded_credentials() {
        assert_eq!(
            redact_upstream_resource_uri(
                "lab://upstream/demo/https://user:pass@example.com/path?token=secret"
            ),
            "lab://upstream/demo/https://example.com/path?token=[redacted]"
        );
    }

    #[test]
    fn redact_upstream_resource_uri_preserves_non_sensitive_query_params() {
        assert_eq!(
            redact_upstream_resource_uri(
                "lab://upstream/demo/https://example.com/items?page=2&limit=50"
            ),
            "lab://upstream/demo/https://example.com/items?page=2&limit=50"
        );
    }

    #[test]
    fn redact_upstream_resource_uri_mixed_query_keys() {
        assert_eq!(
            redact_upstream_resource_uri(
                "lab://upstream/demo/https://example.com/items?page=2&api_key=abc"
            ),
            "lab://upstream/demo/https://example.com/items?page=2&api_key=[redacted]"
        );
    }
}
