//! Action catalog for the `Sonarr` service.
//!
//! This is the single authoritative source. MCP, CLI, and API re-export
//! or reference it.

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
    // ── Series ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "series.list",
        description: "List all TV series in the Sonarr library",
        destructive: false,
        returns: "Series[]",
        params: &[],
    },
    ActionSpec {
        name: "series.get",
        description: "Get a single series by Sonarr ID",
        destructive: false,
        returns: "Series",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Sonarr series ID",
        }],
    },
    ActionSpec {
        name: "series.lookup",
        description: "Search for series to add (TVDB lookup / search term)",
        destructive: false,
        returns: "SeriesLookup[]",
        params: &[ParamSpec {
            name: "query",
            ty: "string",
            required: true,
            description: "Search term or TVDB ID (e.g. 'breaking bad' or 'tvdb:81189')",
        }],
    },
    ActionSpec {
        name: "series.add",
        description: "Add a TV series to Sonarr for monitoring and download",
        destructive: false,
        returns: "Series",
        params: &[
            ParamSpec {
                name: "tvdb_id",
                ty: "i64",
                required: true,
                description: "TVDB ID of the series",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Series title",
            },
            ParamSpec {
                name: "quality_profile_id",
                ty: "i64",
                required: true,
                description: "Quality profile ID (get from qualityprofile.list)",
            },
            ParamSpec {
                name: "language_profile_id",
                ty: "i64",
                required: true,
                description: "Language profile ID (get from languageprofile.list)",
            },
            ParamSpec {
                name: "root_folder_path",
                ty: "string",
                required: true,
                description: "Root folder path (get from rootfolder.list)",
            },
            ParamSpec {
                name: "monitored",
                ty: "bool",
                required: false,
                description: "Monitor series for download (default true)",
            },
            ParamSpec {
                name: "series_type",
                ty: "string",
                required: false,
                description: "Series type: standard, daily, anime (default: standard)",
            },
        ],
    },
    ActionSpec {
        name: "series.delete",
        description: "Delete a series from Sonarr",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Sonarr series ID",
            },
            ParamSpec {
                name: "delete_files",
                ty: "bool",
                required: false,
                description: "Also delete files from disk (default false)",
            },
        ],
    },
    // ── Episodes ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "episode.list",
        description: "List all episodes for a series",
        destructive: false,
        returns: "Episode[]",
        params: &[ParamSpec {
            name: "series_id",
            ty: "i64",
            required: true,
            description: "Sonarr series ID",
        }],
    },
    ActionSpec {
        name: "episode.get",
        description: "Get a single episode by ID",
        destructive: false,
        returns: "Episode",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Sonarr episode ID",
        }],
    },
    // ── Queue ─────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "queue.list",
        description: "List the download queue",
        destructive: false,
        returns: "QueuePage",
        params: &[
            ParamSpec {
                name: "page",
                ty: "u32",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Number of results per page",
            },
            ParamSpec {
                name: "series_id",
                ty: "i64",
                required: false,
                description: "Filter queue to a specific series",
            },
        ],
    },
    ActionSpec {
        name: "queue.delete",
        description: "Remove an item from the download queue",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Queue item ID",
            },
            ParamSpec {
                name: "blocklist",
                ty: "bool",
                required: false,
                description: "Add to blocklist after removal (default false)",
            },
        ],
    },
    // ── History ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "history.list",
        description: "List download history",
        destructive: false,
        returns: "HistoryPage",
        params: &[
            ParamSpec {
                name: "page",
                ty: "u32",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Number of results per page",
            },
            ParamSpec {
                name: "series_id",
                ty: "i64",
                required: false,
                description: "Filter history to a specific series",
            },
            ParamSpec {
                name: "episode_id",
                ty: "i64",
                required: false,
                description: "Filter history to a specific episode",
            },
        ],
    },
    // ── Wanted ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "wanted.list",
        description: "List wanted/missing episodes",
        destructive: false,
        returns: "WantedPage",
        params: &[
            ParamSpec {
                name: "page",
                ty: "u32",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Number of results per page",
            },
        ],
    },
    // ── Calendar ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "calendar.list",
        description: "List upcoming episode air dates",
        destructive: false,
        returns: "Episode[]",
        params: &[
            ParamSpec {
                name: "start",
                ty: "string",
                required: false,
                description: "Start date (ISO 8601, e.g. 2024-01-01)",
            },
            ParamSpec {
                name: "end",
                ty: "string",
                required: false,
                description: "End date (ISO 8601, e.g. 2024-01-31)",
            },
            ParamSpec {
                name: "unmonitored",
                ty: "bool",
                required: false,
                description: "Include unmonitored episodes (default false)",
            },
        ],
    },
    // ── Health & System ───────────────────────────────────────────────────────
    ActionSpec {
        name: "health",
        description: "Return Sonarr health check results",
        destructive: false,
        returns: "HealthCheck[]",
        params: &[],
    },
    ActionSpec {
        name: "system.status",
        description: "Return Sonarr system status and version",
        destructive: false,
        returns: "SystemStatus",
        params: &[],
    },
    // ── Tags ──────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "tag.list",
        description: "List all tags",
        destructive: false,
        returns: "Tag[]",
        params: &[],
    },
    ActionSpec {
        name: "tag.create",
        description: "Create a new tag",
        destructive: false,
        returns: "Tag",
        params: &[ParamSpec {
            name: "label",
            ty: "string",
            required: true,
            description: "Tag label",
        }],
    },
    ActionSpec {
        name: "tag.delete",
        description: "Delete a tag by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Tag ID",
        }],
    },
    // ── Root Folders ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "rootfolder.list",
        description: "List all root folders",
        destructive: false,
        returns: "RootFolder[]",
        params: &[],
    },
    // ── Quality Profiles ──────────────────────────────────────────────────────
    ActionSpec {
        name: "qualityprofile.list",
        description: "List all quality profiles",
        destructive: false,
        returns: "QualityProfile[]",
        params: &[],
    },
    // ── Language Profiles ─────────────────────────────────────────────────────
    ActionSpec {
        name: "languageprofile.list",
        description: "List all language profiles",
        destructive: false,
        returns: "LanguageProfile[]",
        params: &[],
    },
    // ── Series Edit ───────────────────────────────────────────────────────────
    ActionSpec {
        name: "series.edit",
        description: "Update an existing series with a full series resource body",
        destructive: false,
        returns: "Series",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Sonarr series ID",
            },
            ParamSpec {
                name: "body",
                ty: "object",
                required: true,
                description: "Full series resource JSON (obtain via series.get then modify)",
            },
        ],
    },
    // ── Episode Monitor ───────────────────────────────────────────────────────
    ActionSpec {
        name: "episode.monitor",
        description: "Set monitored state for one or more episodes",
        destructive: false,
        returns: "object",
        params: &[
            ParamSpec {
                name: "episode_ids",
                ty: "i64[]",
                required: true,
                description: "List of episode IDs to update",
            },
            ParamSpec {
                name: "monitored",
                ty: "bool",
                required: true,
                description: "Whether to monitor the episodes",
            },
        ],
    },
    // ── Wanted Cutoff ─────────────────────────────────────────────────────────
    ActionSpec {
        name: "wanted.cutoff",
        description: "List episodes that have not met their cutoff quality",
        destructive: false,
        returns: "WantedPage",
        params: &[
            ParamSpec {
                name: "page",
                ty: "u32",
                required: false,
                description: "Page number (1-based)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Number of results per page",
            },
        ],
    },
    // ── Releases ──────────────────────────────────────────────────────────────
    ActionSpec {
        name: "release.search",
        description: "Search for available releases for a series or season",
        destructive: false,
        returns: "Release[]",
        params: &[
            ParamSpec {
                name: "series_id",
                ty: "i64",
                required: false,
                description: "Sonarr series ID to search releases for",
            },
            ParamSpec {
                name: "season_number",
                ty: "i32",
                required: false,
                description: "Season number to search releases for",
            },
        ],
    },
    ActionSpec {
        name: "release.grab",
        description: "Grab a release by GUID and send to download client",
        destructive: false,
        returns: "object",
        params: &[ParamSpec {
            name: "guid",
            ty: "string",
            required: true,
            description: "Release GUID from release.search results",
        }],
    },
    // ── History Series / Failed Retry ─────────────────────────────────────────
    ActionSpec {
        name: "history.series",
        description: "List history records for a specific series",
        destructive: false,
        returns: "HistoryRecord[]",
        params: &[ParamSpec {
            name: "series_id",
            ty: "i64",
            required: true,
            description: "Sonarr series ID",
        }],
    },
    ActionSpec {
        name: "history.failed-retry",
        description: "Retry a failed download by history ID",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "History record ID of the failed download",
        }],
    },
    // ── Blocklist ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "blocklist.list",
        description: "List all blocklisted releases",
        destructive: false,
        returns: "BlocklistPage",
        params: &[],
    },
    ActionSpec {
        name: "blocklist.delete",
        description: "Remove a release from the blocklist by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Blocklist entry ID",
        }],
    },
    // ── Episode File ──────────────────────────────────────────────────────────
    ActionSpec {
        name: "episodefile.delete",
        description: "Delete an episode file from disk by ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Episode file ID",
        }],
    },
    // ── System ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "system.restart",
        description: "Restart the Sonarr application",
        destructive: true,
        returns: "void",
        params: &[],
    },
    ActionSpec {
        name: "system.backup",
        description: "List available system backup files",
        destructive: false,
        returns: "Backup[]",
        params: &[],
    },
];
