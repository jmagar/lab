//! MCP prompt templates.
//!
//! Two prompts for v1:
//! - `run-action` — structured action invocation template
//! - `service-discover` — service exploration starting point

use std::collections::HashMap;

use rmcp::model::{
    GetPromptResult, ListPromptsResult, Prompt, PromptArgument, PromptMessage,
    PromptMessageRole,
};

/// Return all registered prompt templates.
pub fn list_all() -> ListPromptsResult {
    ListPromptsResult::with_all_items(vec![run_action_prompt(), service_discover_prompt()])
}

/// Resolve a prompt by name, interpolating the supplied arguments.
pub fn get(name: &str, args: &HashMap<String, String>) -> Option<GetPromptResult> {
    match name {
        "run-action" => Some(render_run_action(args)),
        "service-discover" => Some(render_service_discover(args)),
        _ => None,
    }
}

// ── Prompt definitions ──────────────────────────────────────────────

fn run_action_prompt() -> Prompt {
    Prompt::new(
        "run-action",
        Some("Execute a lab service action with structured parameters".to_string()),
        Some(vec![
            PromptArgument::new("service").with_description("Service name (e.g. radarr, sonarr)").with_required(true),
            PromptArgument::new("action").with_description("Action to perform (e.g. movie.search)").with_required(true),
            PromptArgument::new("params").with_description("JSON parameters for the action").with_required(false),
        ]),
    )
}

fn service_discover_prompt() -> Prompt {
    Prompt::new(
        "service-discover",
        Some("Explore a lab service's capabilities and available actions".to_string()),
        Some(vec![
            PromptArgument::new("service").with_description("Service name to explore").with_required(true),
        ]),
    )
}

// ── Prompt renderers ────────────────────────────────────────────────

fn render_run_action(args: &HashMap<String, String>) -> GetPromptResult {
    let service = args.get("service").map_or("unknown", |s| s.as_str());
    let action = args.get("action").map_or("help", |s| s.as_str());
    let params = args.get("params").map_or("{}", |s| s.as_str());

    let text = format!(
        "Call the `{service}` service with action `{action}`.\n\
         \n\
         Parameters: {params}\n\
         \n\
         Use the `{service}` tool with:\n\
         ```json\n\
         {{\"action\": \"{action}\", \"params\": {params}}}\n\
         ```"
    );

    GetPromptResult::new(vec![
        PromptMessage::new_text(PromptMessageRole::User, text),
    ])
    .with_description(format!("Run {service}.{action}"))
}

fn render_service_discover(args: &HashMap<String, String>) -> GetPromptResult {
    let service = args.get("service").map_or("unknown", |s| s.as_str());

    let text = format!(
        "I want to explore the `{service}` service.\n\
         \n\
         First, call `{service}` with action `help` to list all available actions.\n\
         Then summarize:\n\
         - What the service does\n\
         - Available actions grouped by resource\n\
         - Which actions are destructive\n\
         - Common workflows"
    );

    GetPromptResult::new(vec![
        PromptMessage::new_text(PromptMessageRole::User, text),
    ])
    .with_description(format!("Discover {service}"))
}
