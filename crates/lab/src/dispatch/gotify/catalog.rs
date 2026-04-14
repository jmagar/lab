use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Single authoritative action catalog for the `Gotify` service.
///
/// MCP, CLI, and API all reference this — never copy the array.
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
    // ── Messages ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "message.send",
        description: "Send a push notification to all subscribers of this app token",
        destructive: false,
        returns: "Message",
        params: &[
            ParamSpec {
                name: "message",
                ty: "string",
                required: true,
                description: "Notification body text",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: false,
                description: "Notification title (optional)",
            },
            ParamSpec {
                name: "priority",
                ty: "integer",
                required: false,
                description: "Priority 0–10; higher values bypass Do Not Disturb",
            },
        ],
    },
    ActionSpec {
        name: "message.list",
        description: "List messages (newest first)",
        destructive: false,
        returns: "PagedMessages",
        params: &[ParamSpec {
            name: "limit",
            ty: "integer",
            required: false,
            description: "Max messages to return (default 100, max 200)",
        }],
    },
    ActionSpec {
        name: "message.delete",
        description: "Delete a single message by id",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Message id",
        }],
    },
    ActionSpec {
        name: "message.purge",
        description: "Delete ALL messages on the server",
        destructive: true,
        returns: "void",
        params: &[],
    },
    // ── Applications ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "app.list",
        description: "List all applications (sending channels)",
        destructive: false,
        returns: "Application[]",
        params: &[],
    },
    ActionSpec {
        name: "app.create",
        description: "Create an application and return its token",
        destructive: false,
        returns: "Application",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Application name",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "Optional description",
            },
        ],
    },
    ActionSpec {
        name: "app.delete",
        description: "Delete an application and all its messages",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Application id",
        }],
    },
    // ── Clients ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "client.list",
        description: "List all clients (receiving subscribers)",
        destructive: false,
        returns: "Client[]",
        params: &[],
    },
    ActionSpec {
        name: "client.create",
        description: "Create a client and return its token",
        destructive: false,
        returns: "Client",
        params: &[ParamSpec {
            name: "name",
            ty: "string",
            required: true,
            description: "Client name",
        }],
    },
    ActionSpec {
        name: "client.delete",
        description: "Delete a client",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Client id",
        }],
    },
    // ── Application (extended) ────────────────────────────────────────────────
    ActionSpec {
        name: "application.update",
        description: "Update an application's name or description",
        destructive: false,
        returns: "Application",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Application id",
            },
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "New application name",
            },
            ParamSpec {
                name: "description",
                ty: "string",
                required: false,
                description: "Optional description",
            },
        ],
    },
    ActionSpec {
        name: "application.messages",
        description: "List all messages for a specific application",
        destructive: false,
        returns: "PagedMessages",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Application id",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Max messages to return",
            },
        ],
    },
    ActionSpec {
        name: "application.messages-delete",
        description: "Delete ALL messages for a specific application",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Application id",
        }],
    },
    // ── Client (extended) ────────────────────────────────────────────────────
    ActionSpec {
        name: "client.update",
        description: "Update a client's name",
        destructive: false,
        returns: "Client",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Client id",
            },
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "New client name",
            },
        ],
    },
    // ── Plugins ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "plugin.list",
        description: "List all server plugins",
        destructive: false,
        returns: "Plugin[]",
        params: &[],
    },
    ActionSpec {
        name: "plugin.enable",
        description: "Enable a plugin",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Plugin id",
        }],
    },
    ActionSpec {
        name: "plugin.disable",
        description: "Disable a plugin",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Plugin id",
        }],
    },
    ActionSpec {
        name: "plugin.config-get",
        description: "Get plugin configuration as YAML text",
        destructive: false,
        returns: "{config: string}",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Plugin id",
        }],
    },
    ActionSpec {
        name: "plugin.config-set",
        description: "Set plugin configuration from a YAML string",
        destructive: false,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Plugin id",
            },
            ParamSpec {
                name: "config",
                ty: "string",
                required: true,
                description: "YAML configuration string",
            },
        ],
    },
    // ── Users ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.list",
        description: "List all users (admin only)",
        destructive: false,
        returns: "User[]",
        params: &[],
    },
    ActionSpec {
        name: "user.create",
        description: "Create a user (admin only)",
        destructive: false,
        returns: "User",
        params: &[
            ParamSpec {
                name: "name",
                ty: "string",
                required: true,
                description: "Username",
            },
            ParamSpec {
                name: "pass",
                ty: "string",
                required: true,
                description: "Password",
            },
            ParamSpec {
                name: "admin",
                ty: "bool",
                required: false,
                description: "Whether the user is an admin (default: false)",
            },
        ],
    },
    ActionSpec {
        name: "user.delete",
        description: "Delete a user (admin only)",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "User id",
        }],
    },
    // ── Server ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "server.health",
        description: "Get server health status (no auth required)",
        destructive: false,
        returns: "Health",
        params: &[],
    },
    ActionSpec {
        name: "server.version",
        description: "Get server version information",
        destructive: false,
        returns: "ServerVersion",
        params: &[],
    },
];
