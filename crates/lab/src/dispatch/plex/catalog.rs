use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `plex` service.
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
    // ── Server ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "server.info",
        description: "Get Plex Media Server identity and configuration",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "server.capabilities",
        description: "Get server media provider capabilities",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── Library ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "library.list",
        description: "List all library sections (Movies, TV Shows, Music, etc.)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "library.get",
        description: "Get metadata for a specific library section",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "section_id",
            ty: "string",
            required: true,
            description: "Library section key/ID",
        }],
    },
    ActionSpec {
        name: "library.scan",
        description: "Trigger a scan of a library section for new content",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "section_id",
            ty: "string",
            required: true,
            description: "Library section key/ID",
        }],
    },
    ActionSpec {
        name: "library.refresh",
        description: "Force a metadata refresh for a library section (re-downloads all metadata)",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "section_id",
            ty: "string",
            required: true,
            description: "Library section key/ID",
        }],
    },
    // ── Media ─────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "media.search",
        description: "Search for media items across libraries",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: true,
                description: "Search query string",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum number of results",
            },
            ParamSpec {
                name: "section_id",
                ty: "string",
                required: false,
                description: "Restrict search to a specific library section",
            },
        ],
    },
    ActionSpec {
        name: "media.get",
        description: "Get metadata for a specific media item by rating key",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key (unique media identifier)",
        }],
    },
    // ── Sessions ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "session.list",
        description: "List all active playback sessions",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "session.terminate",
        description: "Terminate an active playback session",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "session_id",
                ty: "string",
                required: true,
                description: "Session ID to terminate",
            },
            ParamSpec {
                name: "reason",
                ty: "string",
                required: false,
                description: "Reason message shown to the client",
            },
        ],
    },
    // ── Playlists ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "playlist.list",
        description: "List all playlists",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "playlist.get",
        description: "Get a specific playlist by rating key",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "playlist_id",
            ty: "string",
            required: true,
            description: "Playlist rating key",
        }],
    },
    ActionSpec {
        name: "playlist.create",
        description: "Create a new playlist",
        destructive: true,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Playlist title",
            },
            ParamSpec {
                name: "playlist_type",
                ty: "string",
                required: true,
                description: "Playlist type: `video`, `audio`, or `photo`",
            },
            ParamSpec {
                name: "uri",
                ty: "string",
                required: false,
                description: "Library URI to seed the playlist with",
            },
        ],
    },
    ActionSpec {
        name: "playlist.delete",
        description: "Delete a playlist by rating key",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "playlist_id",
            ty: "string",
            required: true,
            description: "Playlist rating key",
        }],
    },
    // ── Library browse ────────────────────────────────────────────────────────
    ActionSpec {
        name: "library.browse",
        description: "List all content in a library section",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "section_id",
                ty: "integer",
                required: true,
                description: "Library section numeric ID",
            },
            ParamSpec {
                name: "type",
                ty: "string",
                required: false,
                description: "Filter by media type (e.g. `\"1\"` movies, `\"4\"` episodes)",
            },
            ParamSpec {
                name: "sort",
                ty: "string",
                required: false,
                description: "Sort field and direction (e.g. `\"titleSort\"`, `\"addedAt:desc\"`)",
            },
        ],
    },
    ActionSpec {
        name: "library.empty-trash",
        description: "Permanently remove deleted items from a library section",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "section_id",
            ty: "integer",
            required: true,
            description: "Library section numeric ID",
        }],
    },
    // ── Metadata ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "metadata.delete",
        description: "Delete a metadata item by rating key",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key (unique media identifier)",
        }],
    },
    ActionSpec {
        name: "metadata.edit",
        description: "Edit metadata fields for an item (pass flat JSON object of key-value pairs)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "rating_key",
                ty: "string",
                required: true,
                description: "Plex rating key",
            },
            ParamSpec {
                name: "fields",
                ty: "object",
                required: true,
                description: "Flat JSON object of metadata fields to update",
            },
        ],
    },
    ActionSpec {
        name: "metadata.refresh",
        description: "Re-download metadata from agents for a specific item",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key",
        }],
    },
    // ── Session history ───────────────────────────────────────────────────────
    ActionSpec {
        name: "session.history",
        description: "Get recently played items (session history)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "account_id",
                ty: "integer",
                required: false,
                description: "Filter history by Plex account ID",
            },
            ParamSpec {
                name: "limit",
                ty: "integer",
                required: false,
                description: "Maximum number of history entries to return",
            },
        ],
    },
    // ── Hubs ──────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "hubs.continue-watching",
        description: "Get Continue Watching hub items",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── Butler ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "butler.list",
        description: "List all butler tasks and their current status",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "butler.run",
        description: "Trigger a specific butler task by name (e.g. `BackupDatabase`)",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "task_name",
            ty: "string",
            required: true,
            description: "Butler task name (e.g. `BackupDatabase`, `CleanOldBundles`)",
        }],
    },
    // ── Scrobble / Unscrobble ─────────────────────────────────────────────────
    ActionSpec {
        name: "item.scrobble",
        description: "Mark a media item as played",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key of the item to mark as played",
        }],
    },
    ActionSpec {
        name: "item.unscrobble",
        description: "Mark a media item as unplayed",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key of the item to mark as unplayed",
        }],
    },
    // ── Updater ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "updater.status",
        description: "Get the current Plex Media Server update status",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── Health ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "health",
        description: "Check server reachability and authentication",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
