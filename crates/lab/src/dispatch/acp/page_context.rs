//! Page-context injection helpers for `session.prompt`.
//!
//! Moved from `api/services/acp.rs` so that MCP and CLI can also benefit from
//! context injection without duplicating the sanitization logic.

/// Maximum tokens allowed for the assembled context prefix.
/// Estimated at ~4 chars/token; we enforce a char budget of 30 * 4 = 120 chars.
const PAGE_CONTEXT_MAX_TOKENS: usize = 30;
const PAGE_CONTEXT_MAX_CHARS: usize = PAGE_CONTEXT_MAX_TOKENS * 4;

/// Characters allowed in pageContext field values.
const PAGE_CONTEXT_ALLOWED: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '/', '_', '-',
];

/// Tokens that indicate prompt-injection attempts — reject the entire pageContext
/// field if any match.
///
/// Only keeps truly dangerous injection terms. Legitimate admin/app route names
/// (`admin`, `prompt`, `system`) are allowed — the character allowlist is the
/// primary safety layer.
const PAGE_CONTEXT_DENY_LIST: &[&str] = &["ignore", "override", "instruction", "assistant"];

/// Optional structured page context sent by the frontend.
/// All fields are strings; validation is applied by `assemble_page_context_prefix`.
pub struct PageContextInput<'a> {
    pub route: &'a str,
    pub entity_type: Option<&'a str>,
    pub entity_id: Option<&'a str>,
}

/// Sanitize a single pageContext field value.
/// Returns `None` if the value fails validation.
/// Strips to allowed characters, truncates to 32 chars, then deny-list checks.
pub fn sanitize_page_context_field(value: &str) -> Option<String> {
    let stripped: String = value
        .chars()
        .filter(|c| PAGE_CONTEXT_ALLOWED.contains(c))
        .take(32)
        .collect();

    if stripped.is_empty() {
        return None;
    }

    // Split on separators before deny-list check to resist bypass.
    let lower = stripped.to_lowercase();
    let segments: Vec<&str> = lower.split(['/', '_', '-']).collect();
    for segment in &segments {
        for denied in PAGE_CONTEXT_DENY_LIST {
            if segment.contains(denied) {
                return None;
            }
        }
    }
    // Also check the full string for runs that span segment boundaries.
    for denied in PAGE_CONTEXT_DENY_LIST {
        if lower.contains(denied) {
            return None;
        }
    }

    Some(stripped)
}

/// Assemble a compact context prefix from validated page context input.
/// Returns `None` if route validation fails.
/// Format: `[context: page={route}]` or `[context: page={route} entity={type}/{id}]`
pub fn assemble_page_context_prefix(
    session_id: &str,
    ctx: &PageContextInput<'_>,
) -> Option<String> {
    let route = sanitize_page_context_field(ctx.route)?;

    let prefix = match (ctx.entity_type, ctx.entity_id) {
        (Some(et), Some(eid)) => {
            let entity_type = sanitize_page_context_field(et)?;
            let entity_id = sanitize_page_context_field(eid)?;
            let candidate = format!("[context: page={route} entity={entity_type}/{entity_id}]");
            if candidate.len() <= PAGE_CONTEXT_MAX_CHARS {
                candidate
            } else {
                let without_id = format!("[context: page={route} entity={entity_type}]");
                if without_id.len() <= PAGE_CONTEXT_MAX_CHARS {
                    without_id
                } else {
                    format!("[context: page={route}]")
                }
            }
        }
        (Some(et), None) => {
            let entity_type = sanitize_page_context_field(et)?;
            format!("[context: page={route} entity={entity_type}]")
        }
        _ => format!("[context: page={route}]"),
    };

    let estimated_tokens = prefix.len().div_ceil(4);
    tracing::info!(
        surface = "dispatch",
        service = "acp",
        action = "session.prompt",
        session_id,
        page_context_route = %route,
        page_context_token_estimate = estimated_tokens,
        "page context injected",
    );

    Some(prefix)
}

/// Build the effective prompt text from optional page context + user prompt.
/// On validation failure the context is silently dropped (never errors the request).
pub fn build_prompt_with_context(
    session_id: &str,
    prompt: &str,
    ctx: Option<&PageContextInput<'_>>,
) -> String {
    if let Some(ctx) = ctx {
        match assemble_page_context_prefix(session_id, ctx) {
            Some(prefix) => format!("{prefix}\n\n{prompt}"),
            None => {
                tracing::warn!(
                    surface = "dispatch",
                    service = "acp",
                    action = "session.prompt",
                    session_id,
                    "page context validation failed — injecting without context",
                );
                prompt.to_string()
            }
        }
    } else {
        prompt.to_string()
    }
}
