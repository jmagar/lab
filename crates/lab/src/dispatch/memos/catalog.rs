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
    ActionSpec {
        name: "user.list",
        description: "List all users (admin only; returns auth_failed for non-admins)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "user.stats",
        description: "Get memo statistics for a user by resource name (e.g. \"users/1\" or \"users/me\")",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "user",
            ty: "string",
            required: true,
            description: "User resource name, e.g. \"users/1\" or \"users/me\"",
        }],
    },
    // ── Webhooks ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "webhook.list",
        description: "List webhooks for a user by resource name",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "user",
            ty: "string",
            required: true,
            description: "User resource name, e.g. \"users/1\" or \"users/me\"",
        }],
    },
    ActionSpec {
        name: "webhook.create",
        description: "Create a webhook for a user",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "user",
                ty: "string",
                required: true,
                description: "User resource name, e.g. \"users/1\" or \"users/me\"",
            },
            ParamSpec {
                name: "url",
                ty: "string",
                required: true,
                description: "Webhook target URL to POST events to",
            },
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Display name for the webhook",
            },
        ],
    },
    // ── Attachments ───────────────────────────────────────────────────────
    ActionSpec {
        name: "attachment.upload",
        description: "Upload a file attachment via multipart/form-data (base64-encoded bytes)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "filename",
                ty: "string",
                required: true,
                description: "Original filename, e.g. \"photo.png\"",
            },
            ParamSpec {
                name: "data_base64",
                ty: "string",
                required: true,
                description: "Base64-encoded file content",
            },
            ParamSpec {
                name: "mime_type",
                ty: "string",
                required: true,
                description: "MIME type, e.g. \"image/png\" or \"application/pdf\"",
            },
        ],
    },
    ActionSpec {
        name: "attachment.delete",
        description: "Delete an attachment by resource name",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Attachment resource name, e.g. \"attachments/123\"",
        }],
    },
    // ── Memo sub-resources ─────────────────────────────────────────────────
    ActionSpec {
        name: "memo.comment-list",
        description: "List comments on a memo",
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
        name: "memo.comment-create",
        description: "Create a comment on a memo",
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
                required: true,
                description: "Comment text content",
            },
        ],
    },
    ActionSpec {
        name: "memo.share-list",
        description: "List share links for a memo",
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
        name: "memo.share-create",
        description: "Create a share link for a memo",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Memo resource name, e.g. \"memos/123\"",
        }],
    },
];
