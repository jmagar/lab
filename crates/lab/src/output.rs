//! Output formatting for CLI commands.
//!
//! All CLI handlers should call [`print`] with their data. It chooses
//! human-readable or JSON output and keeps the styling logic in one place.

#![allow(clippy::print_stdout)]

use std::fmt::Write as _;
use std::io::IsTerminal;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;

/// CLI output format, selected by the top-level `--json` flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text with color and icons when stdout is a TTY.
    #[default]
    Human,
    /// Machine-readable JSON, one value per invocation.
    Json,
}

impl OutputFormat {
    /// Resolve the format from a boolean `--json` flag.
    #[must_use]
    pub const fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

#[derive(Debug, Clone, Copy)]
struct RenderContext {
    color: bool,
}

impl RenderContext {
    fn detect() -> Self {
        let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
        Self { color }
    }
}

/// Print a serializable value in the requested format.
///
/// `Json` emits compact single-line JSON (machine-readable).
/// `Human` emits a structured terminal view with color and icons when
/// stdout supports ANSI.
pub fn print<T: Serialize>(value: &T, format: OutputFormat) -> Result<()> {
    println!("{}", render(value, format)?);
    Ok(())
}

/// Render a serializable value to a string in the requested format.
///
/// Used by [`print`] and available for testing without stdout capture.
pub fn render<T: Serialize>(value: &T, format: OutputFormat) -> Result<String> {
    Ok(match format {
        OutputFormat::Human => render_human(&serde_json::to_value(value)?, RenderContext::detect()),
        OutputFormat::Json => serde_json::to_string(value)?,
    })
}

/// Print a pre-built `tabled::Table` to stdout.
pub fn print_table(table: &tabled::Table) {
    println!("{table}");
}

fn render_human(value: &Value, ctx: RenderContext) -> String {
    match value {
        Value::Object(map) => render_object(map, ctx, 0),
        Value::Array(items) if items.iter().all(Value::is_object) => {
            render_record_array(items, ctx, None)
        }
        Value::Array(items) => render_array(items, ctx, 0),
        _ => render_scalar(value, ctx),
    }
}

fn render_object(
    map: &serde_json::Map<String, Value>,
    ctx: RenderContext,
    indent: usize,
) -> String {
    if is_extract_report(map) {
        return render_extract_report(map, ctx, indent);
    }
    if is_doctor_report(map) {
        return render_doctor_report(map, ctx, indent);
    }

    if map.is_empty() {
        return format!("{}{}", indent_str(indent), dim("∅", ctx));
    }

    let mut lines = Vec::new();
    for (k, v) in map {
        let prefix = indent_str(indent);
        let key = bold(primary(k, ctx), ctx);
        match v {
            Value::Array(items) if items.iter().all(Value::is_object) => {
                lines.push(format!("{}{} {}", prefix, accent("▸", ctx), key));
                lines.push(render_record_array(items, ctx, Some(k)));
            }
            Value::Array(items) => {
                lines.push(format!("{}{} {}", prefix, accent("▸", ctx), key));
                lines.push(render_array(items, ctx, indent + 1));
            }
            Value::Object(child) => {
                lines.push(format!("{}{} {}", prefix, accent("▸", ctx), key));
                lines.push(render_object(child, ctx, indent + 1));
            }
            _ => lines.push(format!(
                "{}{} {} {} {}",
                prefix,
                accent("•", ctx),
                key,
                dim(":", ctx),
                render_scalar(v, ctx)
            )),
        }
    }
    lines.join("\n")
}

fn render_record_array(items: &[Value], ctx: RenderContext, field_name: Option<&str>) -> String {
    if items.is_empty() {
        return dim("∅", ctx);
    }

    if items.iter().all(is_health_row) {
        return render_health_rows(items, ctx);
    }
    if items.iter().all(is_doctor_finding) {
        return render_finding_rows(items, ctx);
    }
    if items.iter().all(is_extract_cred) {
        return render_extract_creds(items, ctx);
    }

    let headers = collect_headers(items);
    let rows = items
        .iter()
        .filter_map(Value::as_object)
        .map(|map| {
            headers
                .iter()
                .map(|header| {
                    map.get(header)
                        .map_or_else(String::new, |value| render_table_cell(header, value, ctx))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut out = String::new();
    if let Some(field_name) = field_name {
        let title = match field_name {
            "creds" => "Credentials",
            "warnings" => "Warnings",
            "findings" => "Findings",
            other => other,
        };
        writeln!(out, "{}", render_heading(title, ctx)).ok();
    }
    out.push_str(&render_table(&headers, &rows, ctx));
    out
}

fn render_table(headers: &[String], rows: &[Vec<String>], ctx: RenderContext) -> String {
    if headers.is_empty() {
        return dim("[]", ctx);
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| visible_width(h)).collect();
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            if idx < widths.len() {
                widths[idx] = widths[idx].max(visible_width(cell));
            }
        }
    }

    let mut out = String::new();
    out.push_str(&render_table_row(headers, &widths, |header| {
        bold(primary(header, ctx), ctx)
    }));
    out.push('\n');
    out.push_str(&render_table_separator(&widths, ctx));

    for row in rows {
        out.push('\n');
        out.push_str(&render_table_row(row, &widths, |cell| cell.clone()));
    }

    out
}

fn render_table_row<F>(cells: &[String], widths: &[usize], map: F) -> String
where
    F: Fn(&String) -> String,
{
    let mut row = String::new();
    for (idx, cell) in cells.iter().enumerate() {
        if idx > 0 {
            row.push(' ');
        }
        let rendered = map(cell);
        let width = widths
            .get(idx)
            .copied()
            .unwrap_or_else(|| visible_width(&rendered));
        row.push_str(&pad_right(&rendered, width));
    }
    row
}

fn render_table_separator(widths: &[usize], ctx: RenderContext) -> String {
    let mut line = String::new();
    for (idx, width) in widths.iter().enumerate() {
        if idx > 0 {
            line.push(' ');
        }
        line.push_str(&subtle(&"━".repeat(*width), ctx));
    }
    line
}

fn render_health_rows(items: &[Value], ctx: RenderContext) -> String {
    let mut ok = 0usize;
    let mut warn = 0usize;
    let mut fail = 0usize;
    let mut rows = Vec::new();

    for item in items.iter().filter_map(Value::as_object) {
        let reachable = item
            .get("reachable")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let auth_ok = item
            .get("auth_ok")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let status = match (reachable, auth_ok) {
            (true, true) => {
                ok += 1;
                status_ok("reachable", ctx)
            }
            (true, false) => {
                warn += 1;
                status_warn("auth", ctx)
            }
            _ => {
                fail += 1;
                status_fail("down", ctx)
            }
        };

        rows.push(vec![
            status,
            item.get("service")
                .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
            bool_icon(auth_ok, ctx),
            item.get("version")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| dim("-", ctx)),
            render_latency(item.get("latency_ms"), ctx),
            item.get("message")
                .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
        ]);
    }

    let headers = vec![
        "Status".to_string(),
        "Service".to_string(),
        "Auth".to_string(),
        "Version".to_string(),
        "Latency".to_string(),
        "Message".to_string(),
    ];

    let mut out = String::new();
    writeln!(
        out,
        "{} {}",
        bold(primary("Service Health", ctx), ctx),
        subtle(format!("({} total)", items.len()).as_str(), ctx)
    )
    .ok();
    writeln!(
        out,
        "{} {} {} {} {} {} {}",
        primary("Status:", ctx),
        status_ok("", ctx),
        accent(ok.to_string().as_str(), ctx),
        status_warn("", ctx),
        accent(warn.to_string().as_str(), ctx),
        status_fail("", ctx),
        accent(fail.to_string().as_str(), ctx)
    )
    .ok();
    out.push('\n');
    out.push_str(&render_table(&headers, &rows, ctx));
    out
}

fn render_finding_rows(items: &[Value], ctx: RenderContext) -> String {
    let mut rows = Vec::new();

    for item in items.iter().filter_map(Value::as_object) {
        let severity = item
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let status = match severity {
            "ok" => status_ok("", ctx),
            "warn" => status_warn("", ctx),
            "fail" => status_fail("", ctx),
            _ => dim("?", ctx),
        };

        rows.push(vec![
            status,
            item.get("service")
                .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
            item.get("check")
                .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
        ]);
    }

    let headers = vec![
        "Status".to_string(),
        "Service".to_string(),
        "Check".to_string(),
    ];

    render_table(&headers, &rows, ctx)
}

fn render_extract_report(
    map: &serde_json::Map<String, Value>,
    ctx: RenderContext,
    _indent: usize,
) -> String {
    let creds = map
        .get("creds")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let warnings = map
        .get("warnings")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let found = map
        .get("found")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut out = String::new();
    writeln!(out, "{}", render_heading("Extract Report", ctx)).ok();
    if let Some(uri) = map.get("uri") {
        writeln!(
            out,
            "{} {}",
            primary("Scan target:", ctx),
            render_uri(uri, ctx)
        )
        .ok();
    }
    writeln!(
        out,
        "{} {} | {} {} | {} {}",
        primary("Found:", ctx),
        accent(found.len().to_string().as_str(), ctx),
        primary("Credentials:", ctx),
        accent(creds.len().to_string().as_str(), ctx),
        primary("Warnings:", ctx),
        accent(warnings.len().to_string().as_str(), ctx)
    )
    .ok();

    if !found.is_empty() {
        let preview = preview_scalar_list(found, 4, ctx);
        writeln!(out, "{} {}", dim("Services:", ctx), preview).ok();
    }

    if !creds.is_empty() {
        out.push('\n');
        out.push_str(&render_extract_creds(creds, ctx));
    }

    if !warnings.is_empty() {
        out.push('\n');
        let headers = vec!["Service".to_string(), "Warning".to_string()];
        let rows = warnings
            .iter()
            .filter_map(Value::as_object)
            .map(|map| {
                vec![
                    map.get("service")
                        .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
                    map.get("message")
                        .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
                ]
            })
            .collect::<Vec<_>>();
        writeln!(out, "{}", render_heading("Warnings", ctx)).ok();
        out.push_str(&render_table(&headers, &rows, ctx));
    }

    out
}

fn render_extract_creds(items: &[Value], ctx: RenderContext) -> String {
    let headers = vec![
        "Service".to_string(),
        "URL".to_string(),
        "Secret".to_string(),
        "Env".to_string(),
    ];
    let rows = items
        .iter()
        .filter_map(Value::as_object)
        .map(|map| {
            vec![
                map.get("service")
                    .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
                map.get("url")
                    .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
                render_secret_state(map.get("secret"), ctx),
                map.get("env_field")
                    .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
            ]
        })
        .collect::<Vec<_>>();
    render_table(&headers, &rows, ctx)
}

fn render_array(items: &[Value], ctx: RenderContext, indent: usize) -> String {
    let prefix = indent_str(indent);
    if items.is_empty() {
        return format!("{prefix}{}", dim("∅", ctx));
    }

    if items.iter().all(is_scalar) && items.len() <= 8 {
        let parts: Vec<String> = items.iter().map(|v| render_scalar(v, ctx)).collect();
        return format!(
            "{prefix}[{}]",
            parts.join(format!(" {} ", accent("·", ctx)).as_str())
        );
    }

    let mut lines = Vec::new();
    for item in items {
        match item {
            Value::Object(map) => {
                lines.push(format!("{prefix}{}", accent("•", ctx)));
                lines.push(render_object(map, ctx, indent + 1));
            }
            Value::Array(child) => {
                lines.push(format!("{prefix}{}", accent("•", ctx)));
                lines.push(render_array(child, ctx, indent + 1));
            }
            _ => lines.push(format!(
                "{prefix}{} {}",
                accent("•", ctx),
                render_scalar(item, ctx)
            )),
        }
    }
    lines.join("\n")
}

fn render_scalar(value: &Value, ctx: RenderContext) -> String {
    match value {
        Value::Null => dim("∅", ctx),
        Value::Bool(true) => status_ok("true", ctx),
        Value::Bool(false) => status_fail("false", ctx),
        Value::Number(n) => accent(&n.to_string(), ctx),
        Value::String(s) => accent(s, ctx),
        other => other.to_string(),
    }
}

fn render_cell_text(value: &Value, ctx: RenderContext) -> String {
    match value {
        Value::Null => dim("-", ctx),
        Value::Bool(true) => bool_icon(true, ctx),
        Value::Bool(false) => bool_icon(false, ctx),
        Value::Number(n) => accent(&n.to_string(), ctx),
        Value::String(s) => accent(s, ctx),
        Value::Array(items) if items.is_empty() => dim("[]", ctx),
        Value::Array(items) => preview_scalar_list(items, 3, ctx),
        Value::Object(map) => format!("{{{} keys}}", map.len()),
    }
}

fn render_table_cell(header: &str, value: &Value, ctx: RenderContext) -> String {
    match header.to_ascii_lowercase().as_str() {
        "status" => render_status_cell(value, ctx),
        "severity" => render_severity_cell(value, ctx),
        "reachable" | "auth_ok" => bool_icon(value.as_bool().unwrap_or(false), ctx),
        "latency_ms" => render_latency(Some(value), ctx),
        "version" => value
            .as_str()
            .filter(|s| !s.is_empty())
            .map_or_else(|| dim("-", ctx), |s| accent(s, ctx)),
        "message" | "warning" | "check" => render_cell_text(value, ctx),
        "secret" => render_secret_state(Some(value), ctx),
        "service" | "name" | "host" => render_cell_text(value, ctx),
        "url" => match value {
            Value::String(s) if !s.is_empty() => accent(s, ctx),
            _ => dim("-", ctx),
        },
        _ => render_cell_text(value, ctx),
    }
}

fn render_status_cell(value: &Value, ctx: RenderContext) -> String {
    if let Some(status) = value.as_str() {
        match status.to_ascii_lowercase().as_str() {
            "running" | "healthy" | "ok" | "reachable" => status_ok("", ctx),
            "partial" | "warn" | "warning" | "degraded" | "auth" => status_warn("", ctx),
            "stopped" | "down" | "fail" | "failed" | "error" | "unreachable" => {
                status_fail("", ctx)
            }
            _ => dim(status, ctx),
        }
    } else if value.as_bool().is_some() {
        bool_icon(value.as_bool().unwrap_or(false), ctx)
    } else {
        render_cell_text(value, ctx)
    }
}

fn render_severity_cell(value: &Value, ctx: RenderContext) -> String {
    match value
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "ok" => status_ok("", ctx),
        "warn" | "warning" => status_warn("", ctx),
        "fail" | "error" => status_fail("", ctx),
        other => dim(other, ctx),
    }
}

fn render_latency(value: Option<&Value>, ctx: RenderContext) -> String {
    let Some(value) = value else {
        return dim("-", ctx);
    };
    let Some(ms) = value.as_u64() else {
        return render_cell_text(value, ctx);
    };
    let text = format!("{ms}ms");
    if ms < 100 {
        accent(&text, ctx)
    } else if ms < 500 {
        paint("33;1", &text, ctx)
    } else {
        primary(&text, ctx)
    }
}

fn render_secret_state(value: Option<&Value>, ctx: RenderContext) -> String {
    match value {
        Some(Value::String(s)) if !s.is_empty() => status_ok("", ctx),
        Some(Value::Bool(true)) => status_ok("", ctx),
        Some(Value::Bool(false)) => status_fail("", ctx),
        Some(Value::Null) | None => dim("-", ctx),
        Some(_) => status_ok("", ctx),
    }
}

fn render_uri(value: &Value, ctx: RenderContext) -> String {
    match value {
        Value::String(s) => accent(s, ctx),
        Value::Object(map) => {
            let host = map.get("host").and_then(Value::as_str);
            let path = map.get("path").and_then(Value::as_str);
            match (host, path) {
                (Some(host), Some(path)) => accent(&format!("{host}:{path}"), ctx),
                _ => render_cell_text(value, ctx),
            }
        }
        _ => render_cell_text(value, ctx),
    }
}

fn preview_scalar_list(items: &[Value], limit: usize, ctx: RenderContext) -> String {
    let mut parts = Vec::new();
    for item in items.iter().take(limit) {
        parts.push(render_cell_text(item, ctx));
    }
    if items.len() > limit {
        parts.push(dim(format!("+{}", items.len() - limit).as_str(), ctx));
    }
    parts.join(", ")
}

fn is_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
    )
}

fn is_health_row(value: &Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.contains_key("service")
        && map.contains_key("reachable")
        && map.contains_key("auth_ok")
        && map.contains_key("latency_ms")
}

fn is_doctor_finding(value: &Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.contains_key("service")
        && map.contains_key("check")
        && map.contains_key("severity")
        && map.contains_key("message")
}

fn is_extract_cred(value: &Value) -> bool {
    let Some(map) = value.as_object() else {
        return false;
    };
    map.contains_key("service")
        && map.contains_key("url")
        && map.contains_key("secret")
        && map.contains_key("env_field")
}

fn is_doctor_report(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("findings")
        && map
            .get("findings")
            .and_then(Value::as_array)
            .map(|items| items.iter().all(is_doctor_finding))
            .unwrap_or(false)
}

fn is_extract_report(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("creds")
        && map
            .get("creds")
            .and_then(Value::as_array)
            .map(|items| items.iter().all(is_extract_cred))
            .unwrap_or(false)
}

fn collect_headers(items: &[Value]) -> Vec<String> {
    let mut headers = Vec::new();
    for item in items {
        if let Some(map) = item.as_object() {
            for key in map.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }
    headers
}

fn render_doctor_report(
    map: &serde_json::Map<String, Value>,
    ctx: RenderContext,
    _indent: usize,
) -> String {
    let findings = map
        .get("findings")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut out = String::new();
    writeln!(out, "{}", render_heading("Doctor Report", ctx)).ok();
    writeln!(
        out,
        "{} {}",
        primary("Checks:", ctx),
        accent(findings.len().to_string().as_str(), ctx)
    )
    .ok();
    if !findings.is_empty() {
        let (ok, warn, fail) = count_findings(findings);
        writeln!(
            out,
            "{} {} {} {} {} {} {}",
            dim("Severity:", ctx),
            status_ok("", ctx),
            accent(ok.to_string().as_str(), ctx),
            status_warn("", ctx),
            accent(warn.to_string().as_str(), ctx),
            status_fail("", ctx),
            accent(fail.to_string().as_str(), ctx)
        )
        .ok();
        out.push('\n');
        out.push_str(&render_finding_rows(findings, ctx));
    }
    out
}

fn count_findings(items: &[Value]) -> (usize, usize, usize) {
    let mut ok = 0usize;
    let mut warn = 0usize;
    let mut fail = 0usize;
    for item in items.iter().filter_map(Value::as_object) {
        match item
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "ok" => ok += 1,
            "warn" | "warning" => warn += 1,
            "fail" | "error" => fail += 1,
            _ => {}
        }
    }
    (ok, warn, fail)
}

fn render_heading(title: &str, ctx: RenderContext) -> String {
    format!(
        "{}\n{}",
        bold(primary(title, ctx), ctx),
        subtle(&"━".repeat(visible_width(title).max(12)), ctx)
    )
}

fn visible_width(text: &str) -> usize {
    strip_ansi_escapes::strip_str(text).chars().count()
}

fn pad_right(text: &str, width: usize) -> String {
    let visible = visible_width(text);
    let mut padded = String::from(text);
    if visible < width {
        padded.push_str(&" ".repeat(width - visible));
    }
    padded
}

fn indent_str(level: usize) -> String {
    "  ".repeat(level)
}

fn status_ok(text: &str, ctx: RenderContext) -> String {
    status_badge("✓", "38;5;39", text, ctx)
}

fn status_warn(text: &str, ctx: RenderContext) -> String {
    status_badge("⚠", "38;5;208", text, ctx)
}

fn status_fail(text: &str, ctx: RenderContext) -> String {
    status_badge("✗", "38;5;160", text, ctx)
}

fn accent(text: &str, ctx: RenderContext) -> String {
    paint("38;5;208", text, ctx)
}

fn primary(text: &str, ctx: RenderContext) -> String {
    paint("38;5;39", text, ctx)
}

fn subtle(text: &str, ctx: RenderContext) -> String {
    paint("38;5;110", text, ctx)
}

fn bold(text: String, ctx: RenderContext) -> String {
    if ctx.color {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text
    }
}

fn dim<T: AsRef<str>>(text: T, ctx: RenderContext) -> String {
    paint("2", text.as_ref(), ctx)
}

fn bool_icon(value: bool, ctx: RenderContext) -> String {
    if value {
        paint("38;5;39", "✓", ctx)
    } else {
        paint("38;5;208", "✗", ctx)
    }
}

fn paint(code: &str, text: &str, ctx: RenderContext) -> String {
    if ctx.color {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn status_badge(icon: &str, code: &str, text: &str, ctx: RenderContext) -> String {
    let _ = text;
    paint(code, icon, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_format_is_compact() {
        let val = json!({"name": "test", "count": 42});
        let out = render(&val, OutputFormat::Json).unwrap();
        assert!(!out.contains('\n'), "Json format must be single-line");
    }

    #[test]
    fn human_format_is_pretty() {
        let val = json!({"name": "test", "count": 42});
        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains('\n'), "Human format must be multi-line");
    }

    #[test]
    fn formats_differ() {
        let val = json!({"a": 1});
        let human = render(&val, OutputFormat::Human).unwrap();
        let json = render(&val, OutputFormat::Json).unwrap();
        assert_ne!(human, json, "Human and Json must produce different output");
    }

    #[test]
    fn from_json_flag() {
        assert_eq!(OutputFormat::from_json_flag(true), OutputFormat::Json);
        assert_eq!(OutputFormat::from_json_flag(false), OutputFormat::Human);
    }

    #[test]
    fn health_rows_render_as_status_table() {
        let val = json!([
            {
                "service": "radarr",
                "reachable": true,
                "auth_ok": true,
                "version": "5.17.2",
                "latency_ms": 41,
                "message": "healthy"
            },
            {
                "service": "unifi",
                "reachable": false,
                "auth_ok": false,
                "version": null,
                "latency_ms": 0,
                "message": "not configured"
            }
        ]);

        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains("Service Health"));
        assert!(out.contains("radarr"));
        assert!(out.contains("unifi"));
        assert!(out.contains("✓"));
        assert!(out.contains("✗"));
    }

    #[test]
    fn doctor_report_renders_summary() {
        let val = json!({
            "findings": [
                {"service": "radarr", "check": "env:RADARR_URL", "severity": "ok", "message": "RADARR_URL is set"},
                {"service": "radarr", "check": "env:RADARR_API_KEY", "severity": "fail", "message": "RADARR_API_KEY is missing"}
            ]
        });

        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains("Doctor Report"));
        assert!(out.contains("Checks:"));
        assert!(out.contains("✓"));
        assert!(out.contains("✗"));
        assert!(!out.contains("ok:"));
        assert!(!out.contains("fail:"));
        assert!(!out.contains("is set"));
        assert!(!out.contains("is missing"));
    }

    #[test]
    fn extract_report_renders_creds_and_warnings() {
        let val = json!({
            "uri": {"Local": "/mnt/appdata"},
            "found": ["radarr", "sonarr", "plex"],
            "creds": [
                {
                    "service": "radarr",
                    "url": "http://radarr:7878",
                    "secret": "abc",
                    "env_field": "RADARR_API_KEY"
                }
            ],
            "warnings": [
                {"service": "plex", "message": "token missing"}
            ]
        });

        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains("Extract Report"));
        assert!(out.contains("Credentials:"));
        assert!(out.contains("Warnings:"));
        assert!(out.contains("radarr"));
        assert!(out.contains("✓"));
        assert!(!out.contains("set"));
    }
}
