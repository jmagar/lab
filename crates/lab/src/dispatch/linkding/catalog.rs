use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Linkding` service.
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
    // ── Bookmarks ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "bookmarks.list",
        description: "List bookmarks with optional search and pagination",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "q",
                ty: "string",
                required: false,
                description: "Search phrase",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum number of results (default 100)",
            },
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Index from which to start returning results",
            },
        ],
    },
    ActionSpec {
        name: "bookmarks.archived.list",
        description: "List archived bookmarks with optional search and pagination",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "q",
                ty: "string",
                required: false,
                description: "Search phrase",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum number of results (default 100)",
            },
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Index from which to start returning results",
            },
        ],
    },
    ActionSpec {
        name: "bookmarks.get",
        description: "Retrieve a single bookmark by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Bookmark ID",
        }],
    },
    ActionSpec {
        name: "bookmarks.check",
        description: "Check whether a URL is already bookmarked",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "url",
            ty: "string",
            required: true,
            description: "URL to check",
        }],
    },
    ActionSpec {
        name: "bookmarks.create",
        description: "Create a new bookmark",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "url",
                ty: "string",
                required: true,
                description: "Bookmark URL",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "Optional title (auto-scraped if omitted)",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "Optional description",
            },
            ParamSpec {
                name: "notes",
                ty: "string",
                required: false,
                description: "Optional markdown notes",
            },
            ParamSpec {
                name: "is_archived",
                ty: "bool",
                required: false,
                description: "Save directly to archive",
            },
            ParamSpec {
                name: "unread",
                ty: "bool",
                required: false,
                description: "Mark as unread",
            },
            ParamSpec {
                name: "shared",
                ty: "bool",
                required: false,
                description: "Share publicly",
            },
            ParamSpec {
                name: "tag_names",
                ty: "json",
                required: false,
                description: "Array of tag names to assign",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Alternative: full JSON body (overrides individual params)",
            },
        ],
    },
    ActionSpec {
        name: "bookmarks.update",
        description: "Partially update a bookmark (PATCH — only provided fields are changed)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Bookmark ID",
            },
            ParamSpec {
                name: "url",
                ty: "string",
                required: false,
                description: "New URL",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "New title",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "New description",
            },
            ParamSpec {
                name: "notes",
                ty: "string",
                required: false,
                description: "New notes",
            },
            ParamSpec {
                name: "unread",
                ty: "bool",
                required: false,
                description: "Update unread status",
            },
            ParamSpec {
                name: "shared",
                ty: "bool",
                required: false,
                description: "Update shared status",
            },
            ParamSpec {
                name: "tag_names",
                ty: "json",
                required: false,
                description: "Replace tag names",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Alternative: full JSON body (overrides individual params)",
            },
        ],
    },
    ActionSpec {
        name: "bookmarks.archive",
        description: "Archive a bookmark",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Bookmark ID",
        }],
    },
    ActionSpec {
        name: "bookmarks.unarchive",
        description: "Unarchive a bookmark",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Bookmark ID",
        }],
    },
    ActionSpec {
        name: "bookmarks.delete",
        description: "Delete a bookmark by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Bookmark ID",
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
    ActionSpec {
        name: "tags.get",
        description: "Retrieve a single tag by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Tag ID",
        }],
    },
    ActionSpec {
        name: "tags.create",
        description: "Create a new tag",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Tag name",
            },
            ParamSpec {
                name: "payload",
                ty: "json",
                required: false,
                description: "Alternative: full JSON body (overrides individual params)",
            },
        ],
    },
    // ── User ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.profile",
        description: "Retrieve user profile and preferences",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
