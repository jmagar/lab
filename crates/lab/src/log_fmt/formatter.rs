use std::collections::BTreeMap;
use std::fmt as stdfmt;

use console::Style;
use jiff::Zoned;

use crate::output::theme::aurora;
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{
    fmt::{
        format::{FormatEvent, FormatFields, Writer},
        FmtContext,
    },
    registry::LookupSpan,
};

// ---------------------------------------------------------------------------
// Field collection
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct EventFieldCollector {
    pub(crate) fields: BTreeMap<&'static str, String>,
}

impl EventFieldCollector {
    fn insert(&mut self, field: &Field, value: String) {
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
// Formatter — mirrors Axon's CliFormat exactly
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub(crate) struct PremiumEventFormatter;

fn write_level(writer: &mut Writer<'_>, level: tracing::Level, ansi: bool) -> stdfmt::Result {
    if ansi {
        let s = match level {
            tracing::Level::ERROR => {
                Style::new().color256(aurora::ERROR).bold().apply_to("ERROR").to_string()
            }
            tracing::Level::WARN => {
                Style::new().color256(aurora::WARN).bold().apply_to(" WARN").to_string()
            }
            tracing::Level::INFO => Style::new().apply_to(" INFO").to_string(),
            tracing::Level::DEBUG => Style::new().dim().apply_to("DEBUG").to_string(),
            tracing::Level::TRACE => Style::new().dim().apply_to("TRACE").to_string(),
        };
        write!(writer, "{s}  ")
    } else {
        let s = match level {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => " WARN",
            tracing::Level::INFO => " INFO",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };
        write!(writer, "{s}  ")
    }
}

/// Semantic accent for structured field values — Aurora palette (ANSI 256).
fn style_value(key: &str, value: &str, level: tracing::Level) -> String {
    match key {
        // SERVICE_NAME pink — service identifiers
        "service" => Style::new().color256(aurora::SERVICE_NAME).apply_to(value).to_string(),
        // ACCENT_PRIMARY blue — names, addresses, routes
        "tool" | "prompt" | "resource_uri" | "upstream" | "route" | "action" | "addr"
        | "instance" | "target" | "capability" => {
            Style::new().color256(aurora::ACCENT_PRIMARY).apply_to(value).to_string()
        }
        // TEXT_MUTED — metadata / phase markers
        "subsystem" | "phase" | "transport" | "operation" => {
            Style::new().color256(aurora::TEXT_MUTED).apply_to(value).to_string()
        }
        // status codes: SUCCESS 2xx, WARN 3xx/4xx, ERROR 5xx
        "status" => {
            if let Ok(n) = value.parse::<u16>() {
                if n < 300 {
                    Style::new().color256(aurora::SUCCESS).apply_to(value).to_string()
                } else if n < 500 {
                    Style::new().color256(aurora::WARN).apply_to(value).to_string()
                } else {
                    Style::new().color256(aurora::ERROR).apply_to(value).to_string()
                }
            } else {
                value.to_string()
            }
        }
        "error" => Style::new().color256(aurora::ERROR).apply_to(value).to_string(),
        "kind" if matches!(level, tracing::Level::WARN | tracing::Level::ERROR) => {
            Style::new().color256(aurora::WARN).apply_to(value).to_string()
        }
        _ => value.to_string(),
    }
}

/// Strip Unicode control characters from upstream-controlled field values to prevent ANSI injection.
/// Tab (0x09) and newline (0x0A) are preserved.
pub(crate) fn sanitize_field_value(value: &str) -> std::borrow::Cow<'_, str> {
    if value.chars().any(|c| c.is_control() && c != '\t' && c != '\n') {
        std::borrow::Cow::Owned(
            value
                .chars()
                .map(|c| if c.is_control() && c != '\t' && c != '\n' { '\u{FFFD}' } else { c })
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
    matches!(
        (key, value),
        ("subject_scoped" | "destructive", "false")
    )
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
        let ansi = writer.has_ansi_escapes();

        let mut fields = EventFieldCollector::default();
        event.record(&mut fields);

        let level = *event.metadata().level();
        let message = fields
            .take("message")
            .map(|m| sanitize_field_value(&m).into_owned())
            .unwrap_or_default();

        // HH:MM:SS (local time, dim)
        let ts = Zoned::now()
            .strftime("%H:%M:%S")
            .to_string();
        if ansi {
            write!(writer, "{}  ", Style::new().dim().apply_to(&ts))?;
        } else {
            write!(writer, "{ts}  ")?;
        }

        // LEVEL
        write_level(&mut writer, level, ansi)?;

        // MESSAGE: first token pink+bold (primary), inline key=val tokens get dim key+eq, rest plain
        if ansi && !message.is_empty() {
            for (i, token) in message.split_whitespace().enumerate() {
                if i > 0 {
                    write!(writer, " ")?;
                }
                if i == 0 {
                    write!(writer, "{}", Style::new().color256(aurora::SERVICE_NAME).bold().apply_to(token))?;
                } else if let Some(eq) = token.find('=') {
                    write!(
                        writer,
                        "{}{}{}",
                        Style::new().dim().apply_to(&token[..eq]),
                        Style::new().dim().apply_to("="),
                        &token[eq + 1..]
                    )?;
                } else {
                    write!(writer, "{token}")?;
                }
            }
        } else {
            write!(writer, "{message}")?;
        }

        // Extra structured fields: two spaces + dim(key) + dim(=) + plain value
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
            "elapsed_ms",
            "error",
        ];

        let write_kv = |writer: &mut Writer<'_>, key: &str, raw: &str| -> stdfmt::Result {
            let safe = sanitize_field_value(raw);
            let formatted = format_field_value(&safe);
            if ansi {
                write!(
                    writer,
                    "  {}{}{}",
                    Style::new().dim().apply_to(key),
                    Style::new().dim().apply_to("="),
                    style_value(key, &formatted, level),
                )
            } else {
                write!(writer, "  {key}={formatted}")
            }
        };

        for key in priority {
            if let Some(val) = fields.take(key) {
                if should_skip_field(key, &val) {
                    continue;
                }
                write_kv(&mut writer, key, &val)?;
            }
        }

        // Remaining fields in alphabetical order (BTreeMap guarantees this).
        let remaining: Vec<_> = fields
            .fields
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        for (key, val) in remaining {
            if should_skip_field(key, &val) {
                continue;
            }
            write_kv(&mut writer, key, &val)?;
        }

        writeln!(writer)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_field_value_quotes_whitespace() {
        assert_eq!(format_field_value("hello world"), r#""hello world""#);
        assert_eq!(format_field_value("nospace"), "nospace");
    }

    #[test]
    fn should_skip_suppresses_false_flags() {
        assert!(should_skip_field("subject_scoped", "false"));
        assert!(should_skip_field("destructive", "false"));
        assert!(!should_skip_field("subject_scoped", "true"));
        assert!(!should_skip_field("error", "false"));
    }

    #[test]
    fn sanitize_strips_c0_controls() {
        let injected = "tool\x1b[31mFAKE";
        let sanitized = sanitize_field_value(injected);
        assert!(!sanitized.contains('\x1b'), "ESC should be replaced");
        assert!(sanitized.contains('\u{FFFD}'), "should contain replacement char");
    }

    #[test]
    fn sanitize_preserves_tab_and_newline() {
        let value = "hello\tworld\nline2";
        let sanitized = sanitize_field_value(value);
        assert_eq!(sanitized, value, "tab and newline must not be replaced");
    }

    #[test]
    fn sanitize_is_noop_for_clean_values() {
        let value = "upstream-name/tool.call";
        let sanitized = sanitize_field_value(value);
        assert!(
            matches!(sanitized, std::borrow::Cow::Borrowed(_)),
            "clean values should borrow"
        );
    }
}
