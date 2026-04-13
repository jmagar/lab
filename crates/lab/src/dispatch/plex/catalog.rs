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
    // ── Health ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "health",
        description: "Check server reachability and authentication",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
