use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Tautulli` service.
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
    // ── Activity ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "activity.list",
        description: "Get current Plex activity (all active streaming sessions)",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "activity.stream",
        description: "Get details for a single active session by session key",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "session_key",
            ty: "string",
            required: true,
            description: "Plex session key",
        }],
    },
    // ── History ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "history.list",
        description: "Get play history with optional filters",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Page number (1-based, default: 1)",
            },
            ParamSpec {
                name: "page_size",
                ty: "integer",
                required: false,
                description: "Results per page (default: 25)",
            },
            ParamSpec {
                name: "order_dir",
                ty: "string",
                required: false,
                description: "Sort direction: asc or desc",
            },
            ParamSpec {
                name: "media_type",
                ty: "string",
                required: false,
                description: "Filter by type: movie, episode, or track",
            },
            ParamSpec {
                name: "user_id",
                ty: "integer",
                required: false,
                description: "Filter by Tautulli user ID",
            },
            ParamSpec {
                name: "section_id",
                ty: "integer",
                required: false,
                description: "Filter by Plex library section ID",
            },
            ParamSpec {
                name: "rating_key",
                ty: "integer",
                required: false,
                description: "Filter by Plex rating key (media item ID)",
            },
        ],
    },
    // ── Users ─────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "users.list",
        description: "List all Tautulli users",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "users.get",
        description: "Get user details by user ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "user_id",
            ty: "integer",
            required: true,
            description: "Tautulli user ID",
        }],
    },
    ActionSpec {
        name: "users.watch_time",
        description: "Get watch time statistics for a user",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "user_id",
            ty: "integer",
            required: true,
            description: "Tautulli user ID",
        }],
    },
    ActionSpec {
        name: "users.player_stats",
        description: "Get player statistics for a user",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "user_id",
            ty: "integer",
            required: true,
            description: "Tautulli user ID",
        }],
    },
    // ── Libraries ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "libraries.list",
        description: "List all Plex libraries tracked by Tautulli",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "libraries.get",
        description: "Get library details by section ID",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "section_id",
            ty: "integer",
            required: true,
            description: "Plex library section ID",
        }],
    },
    ActionSpec {
        name: "libraries.media_info",
        description: "Get media info listing for a library",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "section_id",
            ty: "integer",
            required: true,
            description: "Plex library section ID",
        }],
    },
    // ── Statistics ────────────────────────────────────────────────────────────
    ActionSpec {
        name: "stats.home",
        description: "Get home stats (most played, most active users, recently added)",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "time_range",
                ty: "integer",
                required: false,
                description: "Number of days to include (default: 30)",
            },
            ParamSpec {
                name: "stats_count",
                ty: "integer",
                required: false,
                description: "Number of top results to return (default: 5)",
            },
        ],
    },
    ActionSpec {
        name: "stats.plays_by_date",
        description: "Get play count statistics grouped by date",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "time_range",
                ty: "integer",
                required: false,
                description: "Number of days to include (default: 30)",
            },
            ParamSpec {
                name: "y_axis",
                ty: "string",
                required: false,
                description: "Metric to plot: plays or duration",
            },
        ],
    },
    // ── Media ─────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "media.recently-added",
        description: "Get recently added media items",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "count",
                ty: "integer",
                required: false,
                description: "Number of items to return (default: 5)",
            },
            ParamSpec {
                name: "section_id",
                ty: "string",
                required: false,
                description: "Filter by Plex library section ID",
            },
        ],
    },
    ActionSpec {
        name: "media.metadata",
        description: "Get metadata for a media item by rating key",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key (media item ID)",
        }],
    },
    ActionSpec {
        name: "media.children",
        description: "Get children metadata for a media item by rating key",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "rating_key",
            ty: "string",
            required: true,
            description: "Plex rating key (media item ID)",
        }],
    },
    ActionSpec {
        name: "media.export-metadata",
        description: "Export metadata for a media item",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "rating_key",
                ty: "string",
                required: true,
                description: "Plex rating key (media item ID)",
            },
            ParamSpec {
                name: "media_type",
                ty: "string",
                required: true,
                description: "Media type (movie, show, season, episode, artist, album, track)",
            },
        ],
    },
    // ── User stats ────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.item-stats",
        description: "Get user statistics for a media item by rating key",
        destructive: false,
        returns: "Value",
        params: &[
            ParamSpec {
                name: "rating_key",
                ty: "string",
                required: true,
                description: "Plex rating key (media item ID)",
            },
            ParamSpec {
                name: "media_type",
                ty: "string",
                required: false,
                description: "Media type filter",
            },
        ],
    },
    ActionSpec {
        name: "user.delete-history",
        description: "Delete all play history for a user (permanent)",
        destructive: true,
        returns: "Value",
        params: &[ParamSpec {
            name: "user_id",
            ty: "integer",
            required: true,
            description: "Tautulli user ID whose history will be deleted",
        }],
    },
    // ── Play analytics ────────────────────────────────────────────────────────
    ActionSpec {
        name: "plays.by-day",
        description: "Get play count grouped by day of week",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "time_range",
            ty: "integer",
            required: false,
            description: "Number of days to include (default: 30)",
        }],
    },
    ActionSpec {
        name: "plays.by-hour",
        description: "Get play count grouped by hour of day",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "time_range",
            ty: "integer",
            required: false,
            description: "Number of days to include (default: 30)",
        }],
    },
    ActionSpec {
        name: "plays.by-stream-type",
        description: "Get play count grouped by stream type (transcode vs direct play)",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "time_range",
            ty: "integer",
            required: false,
            description: "Number of days to include (default: 30)",
        }],
    },
    ActionSpec {
        name: "plays.by-month",
        description: "Get play count grouped by month",
        destructive: false,
        returns: "Value",
        params: &[ParamSpec {
            name: "time_range",
            ty: "integer",
            required: false,
            description: "Number of months to include (default: 30)",
        }],
    },
    // ── Server ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "server.pms-update",
        description: "Check for Plex Media Server updates",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    // ── System ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "system.info",
        description: "Get Tautulli server info and status",
        destructive: false,
        returns: "Value",
        params: &[],
    },
    ActionSpec {
        name: "system.settings",
        description: "Get Tautulli settings",
        destructive: false,
        returns: "Value",
        params: &[],
    },
];
