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

use crate::audit::AuditReport;
use crate::scaffold::ScaffoldResult;

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

/// Render a scaffold result for human output.
pub fn render_scaffold_result(result: &ScaffoldResult) -> String {
    let mut out = String::new();

    let mode = if result.dry_run { "dry-run" } else { "applied" };
    writeln!(out, "scaffold {} ({})", result.service, result.kind).ok();
    writeln!(out, "mode: {mode}").ok();

    if result.dry_run {
        writeln!(out, "planned:").ok();
        for op in &result.planned {
            writeln!(out, "  new  {}", op.path.display()).ok();
        }
    } else {
        if !result.created.is_empty() {
            writeln!(out, "created:").ok();
            for path in &result.created {
                writeln!(out, "  - {}", path.display()).ok();
            }
        }
        if !result.modified.is_empty() {
            writeln!(out, "modified:").ok();
            for path in &result.modified {
                writeln!(out, "  - {}", path.display()).ok();
            }
        }
        if result.created.is_empty() && result.modified.is_empty() {
            writeln!(out, "no changes").ok();
        }
    }

    out
}

/// Render an audit report for human output.
pub fn render_audit_report(report: &AuditReport) -> String {
    let mut out = String::new();

    for service in &report.services {
        writeln!(out, "{}:", service.service).ok();
        for (name, result) in &service.checks {
            let status = match result {
                crate::audit::CheckResult::Pass => "pass".to_string(),
                crate::audit::CheckResult::Fail(msg) => format!("fail: {msg}"),
                crate::audit::CheckResult::Skip(msg) => format!("skip: {msg}"),
            };
            writeln!(out, "  - {name}: {status}").ok();
        }
    }

    out
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
#[allow(dead_code)]
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
    if is_catalog(map) {
        return render_catalog(map, ctx);
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
        out.push_str(&render_table_row(row, &widths, Clone::clone));
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
        line.push_str(&subtle(&"─".repeat(*width), ctx));
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
                .map(ToString::to_string)
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
    if let Some(target) = map.get("target") {
        writeln!(
            out,
            "{} {}",
            primary("Scan target:", ctx),
            render_target(target, ctx)
        )
        .ok();
    }
    let verified = creds
        .iter()
        .filter(|cred| {
            cred.as_object()
                .and_then(|map| map.get("url_verified"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    writeln!(
        out,
        "{} {} | {} {} | {} {} | {} {}",
        primary("Found:", ctx),
        accent(found.len().to_string().as_str(), ctx),
        primary("Credentials:", ctx),
        accent(creds.len().to_string().as_str(), ctx),
        primary("Verified:", ctx),
        accent(verified.to_string().as_str(), ctx),
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
        let headers = vec![
            "Service".to_string(),
            "Host".to_string(),
            "Runtime".to_string(),
            "Warning".to_string(),
        ];
        let rows = warnings
            .iter()
            .filter_map(Value::as_object)
            .map(|map| {
                vec![
                    map.get("service")
                        .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
                    map.get("host")
                        .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
                    map.get("runtime")
                        .map_or_else(|| dim("-", ctx), |v| render_runtime(v, ctx)),
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
        "Verified".to_string(),
        "Secret".to_string(),
        "Env".to_string(),
        "Source".to_string(),
        "Probe".to_string(),
        "Runtime".to_string(),
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
                map.get("url_verified")
                    .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
                render_secret_state(map.get("secret"), ctx),
                map.get("env_field")
                    .map_or_else(|| "-".to_string(), |v| render_cell_text(v, ctx)),
                map.get("source_host")
                    .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
                map.get("probe_host")
                    .map_or_else(|| dim("-", ctx), |v| render_cell_text(v, ctx)),
                map.get("runtime")
                    .map_or_else(|| dim("-", ctx), |v| render_runtime(v, ctx)),
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
        "secret" => render_secret_state(Some(value), ctx),
        "url" => match value {
            Value::String(s) if !s.is_empty() => accent(s, ctx),
            _ => dim("-", ctx),
        },
        _ => render_cell_text(value, ctx),
    }
}

#[allow(clippy::option_if_let_else)]
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

fn render_target(value: &Value, ctx: RenderContext) -> String {
    match value {
        Value::Object(map) => match map.get("mode").and_then(Value::as_str) {
            Some("fleet") => accent("fleet", ctx),
            Some("targeted") => map
                .get("uri")
                .map_or_else(|| accent("targeted", ctx), |uri| render_uri(uri, ctx)),
            _ => render_cell_text(value, ctx),
        },
        _ => render_cell_text(value, ctx),
    }
}

fn render_runtime(value: &Value, ctx: RenderContext) -> String {
    let Some(map) = value.as_object() else {
        return render_cell_text(value, ctx);
    };
    let name = map.get("container_name").and_then(Value::as_str);
    let image = map.get("image").and_then(Value::as_str);
    match (name, image) {
        (Some(name), Some(image)) if !image.is_empty() => accent(&format!("{name} ({image})"), ctx),
        (Some(name), _) => accent(name, ctx),
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

const fn is_scalar(value: &Value) -> bool {
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
            .is_some_and(|items| items.iter().all(is_doctor_finding))
}

fn is_extract_report(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("creds")
        && map
            .get("creds")
            .and_then(Value::as_array)
            .is_some_and(|items| items.iter().all(is_extract_cred))
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

    // Group findings by service, preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, (usize, usize, usize)> =
        std::collections::HashMap::new();
    for item in findings.iter().filter_map(Value::as_object) {
        let Some(service) = item.get("service").and_then(Value::as_str) else {
            continue;
        };
        let severity = item
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let entry = groups.entry(service.to_string()).or_insert_with(|| {
            order.push(service.to_string());
            (0, 0, 0)
        });
        match severity.as_str() {
            "ok" => entry.0 += 1,
            "warn" | "warning" => entry.1 += 1,
            "fail" | "error" => entry.2 += 1,
            _ => {}
        }
    }

    let total_services = order.len();
    let mut healthy = 0usize;
    let mut degraded = 0usize;
    let mut unhealthy = 0usize;
    for name in &order {
        let (_ok, warn, fail) = groups[name];
        if fail > 0 {
            unhealthy += 1;
        } else if warn > 0 {
            degraded += 1;
        } else {
            healthy += 1;
        }
    }

    let mut out = String::new();
    writeln!(out, "{}", render_heading("Doctor Report", ctx)).ok();
    writeln!(
        out,
        "{} {} {} {} {} {} {} {}",
        dim(format!("{total_services} services").as_str(), ctx),
        dim("·", ctx),
        status_ok("", ctx),
        dim(format!("{healthy} healthy").as_str(), ctx),
        dim("·", ctx),
        status_warn("", ctx),
        dim(format!("{degraded} degraded").as_str(), ctx),
        if unhealthy > 0 {
            format!(
                "{} {} {}",
                dim("·", ctx),
                status_fail("", ctx),
                dim(format!("{unhealthy} unhealthy").as_str(), ctx)
            )
        } else {
            String::new()
        },
    )
    .ok();

    if order.is_empty() {
        return out;
    }

    out.push('\n');

    // Build one-row-per-service summary.
    let rows: Vec<Vec<String>> = order
        .iter()
        .map(|service| {
            let (ok, warn, fail) = groups[service];
            let status = if fail > 0 {
                status_fail("", ctx)
            } else if warn > 0 {
                status_warn("", ctx)
            } else {
                status_ok("", ctx)
            };
            let total = ok + warn + fail;
            let env_summary = if fail == 0 && warn == 0 {
                dim(format!("env {ok}/{total}").as_str(), ctx)
            } else {
                format!(
                    "{} {}",
                    dim("env", ctx),
                    accent(format!("{ok}/{total}").as_str(), ctx)
                )
            };
            vec![
                format!("  {status}"),
                bold(accent(service, ctx), ctx),
                env_summary,
            ]
        })
        .collect();

    let headers = vec![String::new(), String::new(), String::new()];
    out.push_str(&render_table_plain(&headers, &rows));
    out
}

/// Render a table without headers or separator — used for borderless list layouts.
fn render_table_plain(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|h| visible_width(h)).collect();
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            if idx < widths.len() {
                widths[idx] = widths[idx].max(visible_width(cell));
            } else {
                widths.push(visible_width(cell));
            }
        }
    }
    let mut out = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&render_table_row(row, &widths, Clone::clone));
    }
    out
}

fn is_catalog(map: &serde_json::Map<String, Value>) -> bool {
    map.len() == 1
        && map
            .get("services")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().filter_map(Value::as_object).any(|svc| {
                    svc.contains_key("name")
                        && svc.contains_key("actions")
                        && svc.contains_key("category")
                })
            })
}

fn render_catalog(map: &serde_json::Map<String, Value>, ctx: RenderContext) -> String {
    let services = map
        .get("services")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut out = String::new();
    writeln!(
        out,
        "{} {} {}",
        bold(accent("Lab", ctx), ctx),
        dim("·", ctx),
        dim(format!("{} services", services.len()).as_str(), ctx)
    )
    .ok();
    out.push('\n');

    // Measure service-name column width (cap at 14).
    let name_width = services
        .iter()
        .filter_map(Value::as_object)
        .filter_map(|svc| svc.get("name").and_then(Value::as_str))
        .map(str::len)
        .max()
        .unwrap_or(10)
        .min(14);
    let cat_width = services
        .iter()
        .filter_map(Value::as_object)
        .filter_map(|svc| svc.get("category").and_then(Value::as_str))
        .map(str::len)
        .max()
        .unwrap_or(10)
        .min(14);

    const ACTION_PREVIEW: usize = 5;
    const MAX_ACTIONS_WIDTH: usize = 64;

    for (idx, svc) in services.iter().filter_map(Value::as_object).enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let name = svc.get("name").and_then(Value::as_str).unwrap_or("-");
        let category = svc.get("category").and_then(Value::as_str).unwrap_or("-");
        let status_str = svc.get("status").and_then(Value::as_str).unwrap_or("");
        let actions = svc
            .get("actions")
            .and_then(Value::as_array)
            .map_or(&[][..], Vec::as_slice);
        let status_icon = match status_str {
            "available" => status_ok("", ctx),
            "stub" => status_warn("", ctx),
            _ => status_fail("", ctx),
        };

        writeln!(
            out,
            "  {} {}  {}  {} {}",
            status_icon,
            pad_right(&bold(accent(name, ctx), ctx), name_width),
            pad_right(&dim(category, ctx), cat_width),
            accent(actions.len().to_string().as_str(), ctx),
            dim("actions", ctx),
        )
        .ok();

        // Action preview: first N names, middle-dot separated, truncate at MAX_ACTIONS_WIDTH.
        let names: Vec<&str> = actions
            .iter()
            .filter_map(Value::as_object)
            .filter_map(|a| a.get("name").and_then(Value::as_str))
            .collect();
        if names.is_empty() {
            continue;
        }
        let sep = format!(" {} ", dim("·", ctx));
        let indent = "      ";
        let mut line = String::new();
        let mut shown = 0usize;
        for (i, n) in names.iter().take(ACTION_PREVIEW).enumerate() {
            let candidate = if i == 0 {
                n.to_string()
            } else {
                format!("{sep}{n}")
            };
            if visible_width(&line) + visible_width(&candidate) > MAX_ACTIONS_WIDTH {
                break;
            }
            line.push_str(&candidate);
            shown += 1;
        }
        writeln!(out, "{indent}{line}").ok();
        let remaining = names.len().saturating_sub(shown);
        if remaining > 0 {
            writeln!(
                out,
                "{indent}{}",
                dim(
                    format!("(+{remaining} more — `lab help {name}`)").as_str(),
                    ctx
                )
            )
            .ok();
        }
    }

    out
}

fn render_heading(title: &str, ctx: RenderContext) -> String {
    format!(
        "{}\n{}",
        bold(accent(title, ctx), ctx),
        subtle(&"─".repeat(visible_width(title).max(12)), ctx)
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

// Palette — ANSI 256, 6 colors total. Premium = restraint.
//
// Role      Code            Use
// accent    38;5;39   cyan  service names, section headers
// default   —               data cells (terminal fg)
// dim       38;5;244        labels, hints, secondary text
// muted     38;5;240        rule lines, separators
// success   38;5;78   green ✓ available
// warning   38;5;214  amber ⚠ degraded
// error     38;5;203  red   ✗ unreachable

fn status_ok(text: &str, ctx: RenderContext) -> String {
    status_badge("✓", "38;5;78", text, ctx)
}

fn status_warn(text: &str, ctx: RenderContext) -> String {
    status_badge("⚠", "38;5;214", text, ctx)
}

fn status_fail(text: &str, ctx: RenderContext) -> String {
    status_badge("✗", "38;5;203", text, ctx)
}

fn accent(text: &str, ctx: RenderContext) -> String {
    paint("38;5;39", text, ctx)
}

fn primary(text: &str, ctx: RenderContext) -> String {
    paint("38;5;39", text, ctx)
}

fn subtle(text: &str, ctx: RenderContext) -> String {
    paint("38;5;240", text, ctx)
}

fn bold(text: String, ctx: RenderContext) -> String {
    if ctx.color {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text
    }
}

fn dim<T: AsRef<str>>(text: T, ctx: RenderContext) -> String {
    paint("38;5;244", text.as_ref(), ctx)
}

fn bool_icon(value: bool, ctx: RenderContext) -> String {
    if value {
        paint("38;5;78", "✓", ctx)
    } else {
        paint("38;5;203", "✗", ctx)
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
        // Service-grouped summary: per-service row, not per-check.
        assert!(out.contains("radarr"));
        assert!(out.contains("env"));
        assert!(out.contains("✓"));
        assert!(out.contains("✗"));
        // Per-check detail is hidden in default (grouped) mode.
        assert!(!out.contains("is set"));
        assert!(!out.contains("is missing"));
    }

    #[test]
    fn catalog_renders_nested_layout_without_key_count_artifacts() {
        let val = serde_json::json!({
            "services": [
                {
                    "name": "radarr",
                    "description": "Movie manager",
                    "category": "servarr",
                    "status": "available",
                    "requires_http_subject": false,
                    "actions": [
                        {"name": "movie.search", "description": "", "destructive": false, "params": [], "returns": ""},
                        {"name": "movie.add", "description": "", "destructive": false, "params": [], "returns": ""},
                        {"name": "queue.list", "description": "", "destructive": false, "params": [], "returns": ""},
                        {"name": "queue.purge", "description": "", "destructive": true, "params": [], "returns": ""},
                        {"name": "history.list", "description": "", "destructive": false, "params": [], "returns": ""},
                        {"name": "root.list", "description": "", "destructive": false, "params": [], "returns": ""},
                        {"name": "tag.list", "description": "", "destructive": false, "params": [], "returns": ""}
                    ]
                }
            ]
        });
        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains("Lab"));
        assert!(out.contains("radarr"));
        assert!(out.contains("servarr"));
        assert!(out.contains("7 actions") || out.contains("7\u{1b}[0m actions"));
        assert!(out.contains("movie.search"));
        assert!(out.contains("(+2 more"));
        // The old `{5 keys}` artifact must not appear.
        assert!(
            !out.contains("{5 keys}"),
            "nested ActionEntry rendered as '{{N keys}}' artifact"
        );
        assert!(!out.contains("keys}"), "any 'keys}}' artifact leaked");
    }

    #[test]
    fn extract_report_renders_fleet_provenance_and_url_only_results() {
        let val = json!({
            "target": {"mode": "fleet"},
            "found": ["radarr", "sonarr", "plex"],
            "creds": [
                {
                    "service": "radarr",
                    "url": "http://100.64.0.12:7878",
                    "secret": null,
                    "env_field": "RADARR_API_KEY",
                    "source_host": "media-node",
                    "probe_host": "100.64.0.12",
                    "runtime": {
                        "container_name": "radarr",
                        "image": "lscr.io/linuxserver/radarr:latest"
                    },
                    "url_verified": true
                }
            ],
            "warnings": [
                {
                    "service": "plex",
                    "host": "media-node",
                    "runtime": {
                        "container_name": "plex",
                        "image": "plexinc/pms-docker:latest"
                    },
                    "message": "config root could not be resolved"
                }
            ]
        });

        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains("Extract Report"));
        assert!(out.contains("Scan target:"));
        assert!(out.contains("fleet"));
        assert!(out.contains("Verified:"));
        assert!(out.contains("Source"));
        assert!(out.contains("Probe"));
        assert!(out.contains("Runtime"));
        assert!(out.contains("radarr"));
        assert!(out.contains("media-node"));
        assert!(out.contains("100.64.0.12"));
        assert!(out.contains("lscr.io/linuxserver/radarr:latest"));
        assert!(out.contains("✓"));
        assert!(out.contains("plex"));
        assert!(out.contains("config root could not be resolved"));
    }
}
