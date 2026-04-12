use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Paperless-ngx` service.
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
    // ── Documents ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "documents.list",
        description: "List documents with optional filters",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: false,
                description: "Full-text search query",
            },
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "integer",
                required: false,
                description: "Results per page",
            },
            ParamSpec {
                name: "ordering",
                ty: "string",
                required: false,
                description: "Sort field (prefix with - for descending, e.g. -created)",
            },
            ParamSpec {
                name: "correspondent",
                ty: "integer",
                required: false,
                description: "Filter by correspondent ID",
            },
            ParamSpec {
                name: "document_type",
                ty: "integer",
                required: false,
                description: "Filter by document type ID",
            },
            ParamSpec {
                name: "tags__id__all",
                ty: "string",
                required: false,
                description: "Comma-separated tag IDs that must all be present",
            },
        ],
    },
    ActionSpec {
        name: "documents.get",
        description: "Get a single document by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Document ID",
        }],
    },
    ActionSpec {
        name: "documents.metadata",
        description: "Get full metadata for a document",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Document ID",
        }],
    },
    ActionSpec {
        name: "documents.update",
        description: "Partially update a document (PATCH)",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Document ID",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "New document title",
            },
            ParamSpec {
                name: "correspondent",
                ty: "integer",
                required: false,
                description: "Correspondent ID",
            },
            ParamSpec {
                name: "document_type",
                ty: "integer",
                required: false,
                description: "Document type ID",
            },
            ParamSpec {
                name: "tags",
                ty: "json",
                required: false,
                description: "Array of tag IDs to assign",
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
        name: "documents.delete",
        description: "Delete a document permanently",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Document ID",
        }],
    },
    // ── Tags ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "tags.list",
        description: "List all tags",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "tags.get",
        description: "Get a tag by ID",
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
        description: "Create a tag",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Tag name",
            },
            ParamSpec {
                name: "colour",
                ty: "string",
                required: false,
                description: "Hex colour string (e.g. #ff0000)",
            },
            ParamSpec {
                name: "is_inbox_tag",
                ty: "bool",
                required: false,
                description: "Whether this is the special inbox tag",
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
        name: "tags.delete",
        description: "Delete a tag",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Tag ID",
        }],
    },
    // ── Correspondents ─────────────────────────────────────────────────────
    ActionSpec {
        name: "correspondents.list",
        description: "List all correspondents",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "correspondents.get",
        description: "Get a correspondent by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Correspondent ID",
        }],
    },
    ActionSpec {
        name: "correspondents.create",
        description: "Create a correspondent",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Correspondent name",
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
        name: "correspondents.delete",
        description: "Delete a correspondent",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Correspondent ID",
        }],
    },
    // ── Document Types ──────────────────────────────────────────────────────
    ActionSpec {
        name: "document_types.list",
        description: "List all document types",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "document_types.get",
        description: "Get a document type by ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Document type ID",
        }],
    },
    ActionSpec {
        name: "document_types.create",
        description: "Create a document type",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Document type name",
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
        name: "document_types.delete",
        description: "Delete a document type",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Document type ID",
        }],
    },
    // ── Statistics & Tasks ──────────────────────────────────────────────────
    ActionSpec {
        name: "statistics",
        description: "Get instance statistics (total documents, inbox count, etc.)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "tasks.list",
        description: "List async task status",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
