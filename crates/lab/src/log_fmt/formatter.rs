use std::collections::BTreeMap;
use std::fmt as stdfmt;

use tracing::{Event, Subscriber, field::{Field, Visit}};
use tracing_subscriber::{
    fmt::{FmtContext, format::{FormatEvent, FormatFields, Writer}},
    registry::LookupSpan,
};

// ---------------------------------------------------------------------------
// Field collection
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct EventFieldCollector {
    // &'static str keys: Field::name() already returns &'static str — no key allocation needed.
    pub(crate) fields: BTreeMap<&'static str, String>,
}

impl EventFieldCollector {
    pub(crate) fn insert(&mut self, field: &Field, value: String) {
        self.fields.insert(field.name(), value);
    }

    pub(crate) fn take(&mut self, key: &str) -> Option<String> {
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

// ---------------------------------------------------------------------------
// Formatter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub(crate) struct PremiumEventFormatter {
    pub(crate) ansi: bool,
}

impl PremiumEventFormatter {
    // --- Raw style primitive ---

    fn style<'a>(&self, text: &'a str, code: &str) -> std::borrow::Cow<'a, str> {
        if self.ansi {
            std::borrow::Cow::Owned(format!("\x1b[{code}m{text}\x1b[0m"))
        } else {
            std::borrow::Cow::Borrowed(text)
        }
    }

    // --- Semantic style helpers ---

    fn dim<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "2")
    }

    fn bold<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "1")
    }

    fn badge<'a>(&self, text: &'a str, code: &str) -> std::borrow::Cow<'a, str> {
        self.style(text, code)
    }

    /// Axon accent blue (color256 111) — upstream names, tool names, resource URIs, routes.
    fn accent<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "38;5;111")
    }

    /// Axon primary pink (color256 211) — subject-level identifiers.
    fn subject_accent<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "38;5;211")
    }

    /// Red — error field values.
    fn error_value<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "31")
    }

    /// Yellow — kind field values at WARN/ERROR level.
    fn warn_value<'a>(&self, text: &'a str) -> std::borrow::Cow<'a, str> {
        self.style(text, "33")
    }

    fn level_badge(&self, level: &tracing::Level) -> std::borrow::Cow<'static, str> {
        match *level {
            tracing::Level::ERROR => self.badge("ERROR", "1;31"),
            tracing::Level::WARN  => self.badge("WARN ", "1;33"),
            tracing::Level::INFO  => self.badge("INFO ", "32"),   // Axon green (not bold cyan)
            tracing::Level::DEBUG => self.badge("DEBUG", "1;34"),
            tracing::Level::TRACE => self.badge("TRACE", "1;90"),
        }
    }

    // --- Field rendering helpers ---

    pub(crate) fn normalize_label(value: &str) -> String {
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

    /// Strip C0 control characters (ANSI injection from upstream-controlled field values).
    /// Tab (0x09) and newline (0x0A) are preserved; format_field_value handles whitespace quoting.
    fn sanitize_field_value(value: &str) -> std::borrow::Cow<'_, str> {
        if value.bytes().any(|b| matches!(b, 0x00..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F | 0x7F)) {
            std::borrow::Cow::Owned(
                value
                    .chars()
                    .map(|c| {
                        let n = c as u32;
                        if (n <= 0x1F && c != '\t' && c != '\n') || n == 0x7F {
                            '\u{FFFD}'
                        } else {
                            c
                        }
                    })
                    .collect(),
            )
        } else {
            std::borrow::Cow::Borrowed(value)
        }
    }

    pub(crate) fn format_field_value(value: &str) -> String {
        if value.contains(char::is_whitespace) {
            format!("{value:?}")
        } else {
            value.to_string()
        }
    }

    pub(crate) fn should_skip_field(key: &str, value: &str) -> bool {
        matches!((key, value), ("subject_scoped", "false") | ("destructive", "false"))
    }

    /// Semantic field coloring: accent-tier fields get accent blue; error/kind get red/yellow.
    fn style_field_value<'a>(&self, key: &str, value: &'a str, level: &tracing::Level) -> std::borrow::Cow<'a, str> {
        match key {
            "tool" | "prompt" | "resource_uri" | "upstream" | "route" => self.accent(value),
            "error" => self.error_value(value),
            "kind" if matches!(level, &tracing::Level::WARN | &tracing::Level::ERROR) => {
                self.warn_value(value)
            }
            _ => self.bold(value),
        }
    }

    fn render_extra_fields(
        &self,
        writer: &mut Writer<'_>,
        fields: &mut EventFieldCollector,
        level: &tracing::Level,
    ) -> stdfmt::Result {
        // Runtime-relevant fields rendered first in a stable order.
        // Startup-only fields (session_ttl_secs, http_mcp_enabled, etc.) fall to the catch-all
        // alphabetical loop below with identical output — no visible behavior change.
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
            "method",
            "status",
            "operation",
            "capability",
            "transport",
            "response_bytes",
            "error",
        ];

        let mut rendered_any = false;

        let mut render_kv = |writer: &mut Writer<'_>, key: &str, value: &str| -> stdfmt::Result {
            if !rendered_any {
                write!(writer, " {} ", self.dim("|"))?;
                rendered_any = true;
            } else {
                write!(writer, " ")?;
            }
            let safe = Self::sanitize_field_value(value);
            let formatted = Self::format_field_value(&safe);
            write!(
                writer,
                "{}={}",
                self.dim(key),
                self.style_field_value(key, &formatted, level),
            )
        };

        for key in priority {
            if let Some(value) = fields.take(key) {
                if Self::should_skip_field(key, &value) {
                    continue;
                }
                render_kv(writer, key, &value)?;
            }
        }

        // Remaining fields in alphabetical order (BTreeMap guarantees this).
        let remaining: Vec<_> = fields.fields.iter().map(|(k, v)| (*k, v.clone())).collect();
        for (key, value) in remaining {
            if Self::should_skip_field(key, &value) {
                continue;
            }
            render_kv(writer, key, &value)?;
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

        let level = event.metadata().level();
        let message = fields
            .take("message")
            .map(|m| Self::sanitize_field_value(&m).into_owned())
            .unwrap_or_default();
        let (lane, use_target_fallback) =
            Self::render_lane(&mut fields, event.metadata().target());
        let subject =
            Self::render_subject(&mut fields, event.metadata().target(), use_target_fallback);
        let elapsed_ms = fields.take("elapsed_ms");

        write!(writer, "{} {}", self.level_badge(level), self.bold(&lane))?;

        if !subject.is_empty() {
            write!(writer, " {}", self.subject_accent(&subject))?;
        }

        if let Some(elapsed_ms) = elapsed_ms {
            write!(writer, " {}", self.badge(&format!("{elapsed_ms}ms"), "1;32"))?;
        }

        if !message.is_empty() {
            write!(writer, " {}", self.dim("-"))?;
            write!(writer, " {message}")?;
        }

        self.render_extra_fields(&mut writer, &mut fields, level)?;
        writeln!(writer)
    }
}

// ---------------------------------------------------------------------------
// Unit tests for pure helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_label_replaces_underscores_and_uppercases() {
        assert_eq!(PremiumEventFormatter::normalize_label("api_server"), "API-SERVER");
        assert_eq!(PremiumEventFormatter::normalize_label("gateway_client"), "GATEWAY-CLIENT");
    }

    #[test]
    fn format_field_value_quotes_whitespace() {
        assert_eq!(PremiumEventFormatter::format_field_value("hello world"), r#""hello world""#);
        assert_eq!(PremiumEventFormatter::format_field_value("nospace"), "nospace");
    }

    #[test]
    fn should_skip_suppresses_false_flags() {
        assert!(PremiumEventFormatter::should_skip_field("subject_scoped", "false"));
        assert!(PremiumEventFormatter::should_skip_field("destructive", "false"));
        assert!(!PremiumEventFormatter::should_skip_field("subject_scoped", "true"));
        assert!(!PremiumEventFormatter::should_skip_field("error", "false"));
    }

    #[test]
    fn sanitize_strips_c0_controls() {
        let injected = "tool\x1b[31mFAKE";
        let sanitized = PremiumEventFormatter::sanitize_field_value(injected);
        assert!(!sanitized.contains('\x1b'), "ESC should be replaced");
        assert!(sanitized.contains('\u{FFFD}'), "should contain replacement char");
    }

    #[test]
    fn sanitize_preserves_tab_and_newline() {
        let value = "hello\tworld\nline2";
        let sanitized = PremiumEventFormatter::sanitize_field_value(value);
        assert_eq!(sanitized, value, "tab and newline must not be replaced");
    }

    #[test]
    fn sanitize_is_noop_for_clean_values() {
        let value = "upstream-name/tool.call";
        let sanitized = PremiumEventFormatter::sanitize_field_value(value);
        assert!(matches!(sanitized, std::borrow::Cow::Borrowed(_)), "clean values should borrow");
    }
}
