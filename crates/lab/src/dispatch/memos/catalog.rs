use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Memos` service.
///
/// This is the single authoritative source. MCP, CLI, and API re-export
/// or reference it.
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
    // ── Memos ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "memos.list",
        description: "List memos, optionally filtered by creator, tag, or paginated",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "creator",
                ty: "string",
                required: false,
                description: "Filter by creator resource name, e.g. \"users/1\"",
            },
            ParamSpec {
                name: "tag",
                ty: "string",
                required: false,
                description: "Filter by tag name",
            },
            ParamSpec {
                name: "page_size",
                ty: "integer",
                required: false,
                description: "Maximum number of memos to return per page",
            },
            ParamSpec {
                name: "page_token",
                ty: "string",
                required: false,
                description: "Page token from a previous list response",
            },
        ],
    },
    ActionSpec {
        name: "memos.get",
        description: "Get a single memo by resource name (e.g. \"memos/123\")",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Memo resource name, e.g. \"memos/123\"",
        }],
    },
    ActionSpec {
        name: "memos.create",
        description: "Create a new memo",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "content",
                ty: "string",
                required: true,
                description: "Markdown content of the memo",
            },
            ParamSpec {
                name: "visibility",
                ty: "string",
                required: false,
                description: "Visibility: PRIVATE (default), PROTECTED, or PUBLIC",
            },
        ],
    },
    ActionSpec {
        name: "memos.update",
        description: "Update a memo by resource name (PATCH — only provided fields are changed)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Memo resource name, e.g. \"memos/123\"",
            },
            ParamSpec {
                name: "content",
                ty: "string",
                required: false,
                description: "New markdown content",
            },
            ParamSpec {
                name: "visibility",
                ty: "string",
                required: false,
                description: "New visibility: PRIVATE, PROTECTED, or PUBLIC",
            },
        ],
    },
    ActionSpec {
        name: "memos.delete",
        description: "Delete a memo by resource name",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Memo resource name, e.g. \"memos/123\"",
        }],
    },
    // ── Tags ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "tags.list",
        description: "List all tags",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── Workspace ─────────────────────────────────────────────────────────
    ActionSpec {
        name: "workspace.profile",
        description: "Get workspace profile (name, version, owner)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── User ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.me",
        description: "Get the authenticated user's profile",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
