//! Action catalog for the `acp` (Agent Client Protocol) service.
//!
//! Single authoritative source for MCP, CLI, and API adapters.
//! `session.cancel` and `session.close` are marked destructive.

use lab_apis::core::action::{ActionSpec, ParamSpec};

pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        returns: "Schema",
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
    },
    ActionSpec {
        name: "provider.list",
        description: "List available providers with health status",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "provider.get",
        description: "Get one provider's health and capabilities",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "provider",
            ty: "string",
            required: true,
            description: "Provider name (e.g. 'codex')",
        }],
    },
    ActionSpec {
        name: "provider.select",
        description: "Set the default provider for new sessions",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "provider",
            ty: "string",
            required: true,
            description: "Provider name to set as default",
        }],
    },
    ActionSpec {
        name: "session.list",
        description: "List all sessions owned by the caller",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "principal",
            ty: "string",
            required: false,
            description: "Filter sessions by principal (defaults to caller identity)",
        }],
    },
    ActionSpec {
        name: "session.get",
        description: "Get one session's summary and state",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "session_id",
            ty: "string",
            required: true,
            description: "Session ID to retrieve",
        }],
    },
    ActionSpec {
        name: "session.start",
        description: "Create and start a new agent session",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "provider",
                ty: "string",
                required: false,
                description: "Provider to use (default: 'codex')",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "Human-readable session title",
            },
            ParamSpec {
                name: "cwd",
                ty: "string",
                required: false,
                description: "Working directory for the session",
            },
            ParamSpec {
                name: "principal",
                ty: "string",
                required: false,
                description: "Caller principal (defaults to empty = anonymous)",
            },
        ],
    },
    ActionSpec {
        name: "session.prompt",
        description: "Send a prompt to a running session",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Target session ID",
            },
            ParamSpec {
                name: "text",
                ty: "string",
                required: true,
                description: "Prompt text to send",
            },
            ParamSpec {
                name: "principal",
                ty: "string",
                required: false,
                description: "Caller principal for ownership verification",
            },
            ParamSpec {
                name: "page_context",
                ty: "object",
                required: false,
                description: "Optional page context: {route, entityType?, entityId?} — prepends a compact context prefix to the prompt",
            },
        ],
    },
    ActionSpec {
        name: "session.cancel",
        description: "Cancel a running session [destructive]",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Session ID to cancel",
            },
            ParamSpec {
                name: "principal",
                ty: "string",
                required: false,
                description: "Caller principal for ownership verification",
            },
        ],
    },
    ActionSpec {
        name: "session.close",
        description: "Close a session permanently [destructive]",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Session ID to close",
            },
            ParamSpec {
                name: "principal",
                ty: "string",
                required: false,
                description: "Caller principal for ownership verification",
            },
        ],
    },
    ActionSpec {
        name: "session.events",
        description: "Get stored events for a session",
        destructive: false,
        returns: "Vec<AcpEvent> — ProviderInfo events of type 'tool_call_metadata' carry \
                 an optional '_meta' object relayed transparently from the originating agent; \
                 key is absent (not null) when the agent did not inject it. \
                 ToolCallUpdate events carry merged '_meta' (outer wrapper wins over raw_output).",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Session ID to fetch events for",
            },
            ParamSpec {
                name: "since",
                ty: "integer",
                required: false,
                description: "Return events after this sequence number (default 0)",
            },
        ],
    },
    ActionSpec {
        name: "session.subscribe_ticket",
        description: "Issue a short-lived SSE auth ticket for browser EventSource clients",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Session ID to subscribe to",
            },
            ParamSpec {
                name: "principal",
                ty: "string",
                required: false,
                description: "Caller principal for ownership verification",
            },
        ],
    },
];
