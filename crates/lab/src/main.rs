//! `lab` binary entry point.
//!
//! Initializes tracing, loads config, parses clap args, and dispatches
//! to the appropriate subcommand handler. All subsystems are sibling
//! modules declared here.

#![allow(clippy::multiple_crate_versions)]
#![allow(unreachable_pub)] // binary crate — `pub` items are crate-internal by design

mod api;
mod audit;
mod catalog;
mod cli;
mod config;
mod device;
mod dispatch;
mod mcp;
mod oauth;
mod output;
mod registry;
mod scaffold;
#[cfg(test)]
mod test_support;
mod tui;

use std::process::ExitCode;
use std::{collections::BTreeMap, fmt as stdfmt};

use clap::Parser;
use is_terminal::IsTerminal;
use tracing::{Event, Subscriber, field::{Field, Visit}};
use tracing_subscriber::{
    EnvFilter,
    filter::filter_fn,
    fmt::{self, FmtContext, format::{FormatEvent, FormatFields, Writer}},
    prelude::*,
    registry::LookupSpan,
};

use crate::cli::Cli;
use crate::dispatch::logs::ingest::LogIngestLayer;

#[derive(Default)]
struct EventFieldCollector {
    fields: BTreeMap<String, String>,
}

impl EventFieldCollector {
    fn insert(&mut self, field: &Field, value: String) {
        self.fields.insert(field.name().to_string(), value);
    }

    fn take(&mut self, key: &str) -> Option<String> {
        self.fields.remove(key)
    }
}

impl Visit for EventFieldCollector {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.insert(field, format!("{value:?}"));
    }
}

#[derive(Clone, Copy)]
struct PremiumEventFormatter {
    ansi: bool,
}

impl PremiumEventFormatter {
    fn style<'a>(&self, text: &'a str, code: &str) -> std::borrow::Cow<'a, str> {
        if self.ansi {
            std::borrow::Cow::Owned(format!("\x1b[{code}m{text}\x1b[0m"))
        } else {
            std::borrow::Cow::Borrowed(text)
        }
    }

    fn dim<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "2")
    }

    fn bold<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "1")
    }

    fn badge<'a>(&self, text: &'a str, color: &str) -> std::borrow::Cow<'a, str> {
        self.style(text, color)
    }

    fn level_badge(&self, level: &tracing::Level) -> std::borrow::Cow<'static, str> {
        match *level {
            tracing::Level::ERROR => self.badge("ERROR", "1;31"),
            tracing::Level::WARN => self.badge("WARN ", "1;33"),
            tracing::Level::INFO => self.badge("INFO ", "1;36"),
            tracing::Level::DEBUG => self.badge("DEBUG", "1;34"),
            tracing::Level::TRACE => self.badge("TRACE", "1;90"),
        }
    }

    fn normalize_label(value: &str) -> String {
        value.replace('_', "-").to_ascii_uppercase()
    }

    fn render_subject(
        fields: &mut EventFieldCollector,
        target: &str,
        use_target_fallback: bool,
    ) -> String {
        let service = fields.take("service");
        let action = fields.take("action");
        match (service, action) {
            (Some(service), Some(action)) if !action.is_empty() => format!("{service}.{action}"),
            (Some(service), _) => service,
            (None, Some(action)) if !action.is_empty() => action,
            _ if use_target_fallback => target.to_string(),
            _ => String::new(),
        }
    }

    fn render_lane(fields: &mut EventFieldCollector, target: &str) -> (String, bool) {
        if let Some(subsystem) = fields.take("subsystem") {
            let mut lane = Self::normalize_label(&subsystem);
            if let Some(phase) = fields.take("phase") {
                lane.push(' ');
                lane.push_str(&phase);
            }
            (lane, false)
        } else if let Some(surface) = fields.take("surface") {
            (Self::normalize_label(&surface), false)
        } else {
            (Self::normalize_label(target), true)
        }
    }

    fn format_field_value(value: &str) -> String {
        if value.contains(char::is_whitespace) {
            format!("{value:?}")
        } else {
            value.to_string()
        }
    }

    fn should_skip_field(key: &str, value: &str) -> bool {
        matches!((key, value), ("subject_scoped", "false") | ("destructive", "false"))
    }

    fn render_extra_fields(
        &self,
        writer: &mut Writer<'_>,
        fields: &mut EventFieldCollector,
    ) -> stdfmt::Result {
        let priority = [
            "kind",
            "request_id",
            "tool",
            "prompt",
            "resource_uri",
            "upstream",
            "route",
            "instance",
            "addr",
            "path",
            "services",
            "peer_count",
            "method",
            "status",
            "operation",
            "capability",
            "transport",
            "response_bytes",
            "max_bytes",
            "service_count",
            "selected_service_count",
            "requested_service_count",
            "param_key_count",
            "upstream_count",
            "discovered_upstream_count",
            "oauth_upstream_count",
            "allowed_host_count",
            "cors_origin_count",
            "session_ttl_secs",
            "auth_mode",
            "bind_host",
            "bind_port",
            "device_role",
            "stateful",
            "http_mcp_enabled",
            "mcp_server_enabled",
            "web_server_enabled",
            "web_ui_auth_disabled",
            "public_url_configured",
            "bearer_token_configured",
            "local_host",
            "master_host",
            "tools_changed",
            "resources_changed",
            "prompts_changed",
            "error",
            "level",
        ];

        let mut rendered_any = false;
        for key in priority {
            if let Some(value) = fields.take(key) {
                if Self::should_skip_field(key, &value) {
                    continue;
                }
                if !rendered_any {
                    write!(writer, " {} ", self.dim("|"))?;
                    rendered_any = true;
                } else {
                    write!(writer, " ")?;
                }
                write!(
                    writer,
                    "{}={}",
                    self.dim(key),
                    self.bold(&Self::format_field_value(&value))
                )?;
            }
        }

        for (key, value) in &fields.fields {
            if Self::should_skip_field(key, value) {
                continue;
            }
            if !rendered_any {
                write!(writer, " {} ", self.dim("|"))?;
                rendered_any = true;
            } else {
                write!(writer, " ")?;
            }
            write!(
                writer,
                "{}={}",
                self.dim(key),
                self.bold(&Self::format_field_value(value))
            )?;
        }

        Ok(())
    }
}

impl<S, N> FormatEvent<S, N> for PremiumEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> stdfmt::Result {
        let mut fields = EventFieldCollector::default();
        event.record(&mut fields);

        let message = fields.take("message").unwrap_or_default();
        let (lane, use_target_fallback) = Self::render_lane(&mut fields, event.metadata().target());
        let subject = Self::render_subject(&mut fields, event.metadata().target(), use_target_fallback);
        let elapsed_ms = fields.take("elapsed_ms");

        write!(writer, "{} {}", self.level_badge(event.metadata().level()), self.bold(&lane))?;

        if !subject.is_empty() {
            write!(writer, " {}", subject)?;
        }

        if let Some(elapsed_ms) = elapsed_ms {
            write!(writer, " {}", self.badge(&format!("{elapsed_ms}ms"), "1;32"))?;
        }

        if !message.is_empty() {
            write!(writer, " {}", self.dim("-"))?;
            write!(writer, " {}", message)?;
        }

        self.render_extra_fields(&mut writer, &mut fields)?;
        writeln!(writer)
    }
}

fn human_logs_use_ansi() -> bool {
    std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn human_console_target_enabled(target: &str) -> bool {
    target == "lab"
        || target.starts_with("lab::")
        || target == "lab_apis"
        || target.starts_with("lab_apis::")
        || target == "lab_auth"
        || target.starts_with("lab_auth::")
}

/// Initialize tracing.
///
/// Accepts config.toml log preferences; env vars `LAB_LOG` / `LAB_LOG_FORMAT`
/// override them when set.
fn init_tracing(log: &config::LogPreferences) {
    // Env var wins → config.toml → default.
    let filter = EnvFilter::try_from_env("LAB_LOG").unwrap_or_else(|_| {
        let directive = log
            .filter
            .as_deref()
            .unwrap_or("lab=info,lab_apis=warn,rmcp=warn");
        EnvFilter::new(directive)
    });

    let use_json = match std::env::var("LAB_LOG_FORMAT").ok() {
        Some(v) => v.eq_ignore_ascii_case("json"),
        None => log
            .format
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("json")),
    };

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(LogIngestLayer)
            .with(fmt::layer().json().with_writer(std::io::stderr))
            .init();
    } else {
        let ansi = human_logs_use_ansi();
        let fmt_layer = fmt::layer()
            .with_target(false)
            .event_format(PremiumEventFormatter { ansi })
            .with_ansi(ansi)
            .with_writer(std::io::stderr)
            .with_filter(filter_fn(|metadata| human_console_target_enabled(metadata.target())));
        tracing_subscriber::registry()
            .with(filter)
            .with(LogIngestLayer)
            .with(fmt_layer)
            .init();
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    // 1. Load config.toml first (lightweight, no tracing needed).
    //    eprintln is intentional — tracing isn't initialized yet.
    let config = match config::load_toml(&config::toml_candidates()) {
        Ok(cfg) => cfg,
        Err(err) => {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("config.toml parse error: {err:#}");
            }
            return ExitCode::from(2);
        }
    };

    // 2. Init tracing using config.toml [log] preferences (env vars override).
    init_tracing(&config.log);

    // 3. Load .env files (secrets + URL env vars).
    if let Err(err) = config::load_dotenv() {
        tracing::error!("dotenv load error: {err:#}");
        return ExitCode::from(2);
    }

    let cli = Cli::parse();

    match cli::dispatch(cli, config).await {
        Ok(code) => code,
        Err(err) => {
            tracing::error!("{err:#}");
            ExitCode::from(1)
        }
    }
}
