use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `Overseerr` service.
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
    // ── Status / Health ───────────────────────────────────────────────────────
    ActionSpec {
        name: "health",
        description: "Probe Overseerr reachability and auth (lightweight ping)",
        destructive: false,
        returns: "HealthStatus",
        params: &[],
    },
    ActionSpec {
        name: "status",
        description: "Get current Overseerr status (version, update info)",
        destructive: false,
        returns: "OverseerrStatus",
        params: &[],
    },
    // ── Requests ─────────────────────────────────────────────────────────────
    ActionSpec {
        name: "request.list",
        description: "List media requests with optional filters and pagination",
        destructive: false,
        returns: "RequestList",
        params: &[
            ParamSpec {
                name: "take",
                ty: "integer",
                required: false,
                description: "Number of results to return (default 20)",
            },
            ParamSpec {
                name: "skip",
                ty: "integer",
                required: false,
                description: "Number of results to skip (for pagination)",
            },
            ParamSpec {
                name: "filter",
                ty: "string",
                required: false,
                description: "Filter by status: all|approved|available|pending|processing|unavailable|failed",
            },
            ParamSpec {
                name: "sort",
                ty: "string",
                required: false,
                description: "Sort order: added|modified",
            },
            ParamSpec {
                name: "requested_by",
                ty: "integer",
                required: false,
                description: "Filter by requesting user ID",
            },
        ],
    },
    ActionSpec {
        name: "request.get",
        description: "Get a single request by ID",
        destructive: false,
        returns: "MediaRequest",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Request ID",
        }],
    },
    ActionSpec {
        name: "request.create",
        description: "Create a new media request for a movie or TV show",
        destructive: false,
        returns: "MediaRequest",
        params: &[
            ParamSpec {
                name: "media_type",
                ty: "string",
                required: true,
                description: "Media type: movie or tv",
            },
            ParamSpec {
                name: "media_id",
                ty: "integer",
                required: true,
                description: "TMDB media ID",
            },
            ParamSpec {
                name: "seasons",
                ty: "string",
                required: false,
                description: "Comma-separated season numbers for TV requests (e.g. '1,2,3')",
            },
            ParamSpec {
                name: "is4k",
                ty: "bool",
                required: false,
                description: "Request in 4K quality (default false)",
            },
        ],
    },
    ActionSpec {
        name: "request.approve",
        description: "Approve a pending media request",
        destructive: false,
        returns: "MediaRequest",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Request ID to approve",
        }],
    },
    ActionSpec {
        name: "request.decline",
        description: "Decline a pending media request",
        destructive: false,
        returns: "MediaRequest",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Request ID to decline",
        }],
    },
    ActionSpec {
        name: "request.delete",
        description: "Delete a request permanently",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Request ID to delete",
        }],
    },
    // ── Search ───────────────────────────────────────────────────────────────
    ActionSpec {
        name: "movie.search",
        description: "Search for movies by title or keywords",
        destructive: false,
        returns: "SearchResponse",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: true,
                description: "Search query",
            },
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Page number (default 1)",
            },
        ],
    },
    ActionSpec {
        name: "tv.search",
        description: "Search for TV shows by title or keywords",
        destructive: false,
        returns: "SearchResponse",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: true,
                description: "Search query",
            },
            ParamSpec {
                name: "page",
                ty: "integer",
                required: false,
                description: "Page number (default 1)",
            },
        ],
    },
    // ── Movie details ─────────────────────────────────────────────────────────
    ActionSpec {
        name: "movie.get",
        description: "Get detailed information for a movie by TMDB ID",
        destructive: false,
        returns: "MovieDetail",
        params: &[ParamSpec {
            name: "tmdb_id",
            ty: "integer",
            required: true,
            description: "TMDB movie ID",
        }],
    },
    // ── TV details ────────────────────────────────────────────────────────────
    ActionSpec {
        name: "tv.get",
        description: "Get detailed information for a TV show by TMDB ID",
        destructive: false,
        returns: "TvDetail",
        params: &[ParamSpec {
            name: "tmdb_id",
            ty: "integer",
            required: true,
            description: "TMDB TV show ID",
        }],
    },
    // ── Users ─────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "user.list",
        description: "List all Overseerr users with pagination",
        destructive: false,
        returns: "UserList",
        params: &[
            ParamSpec {
                name: "take",
                ty: "integer",
                required: false,
                description: "Number of users to return",
            },
            ParamSpec {
                name: "skip",
                ty: "integer",
                required: false,
                description: "Number of users to skip",
            },
        ],
    },
    ActionSpec {
        name: "user.get",
        description: "Get a single user by ID",
        destructive: false,
        returns: "User",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "User ID",
        }],
    },
    // ── Issues ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "issue.list",
        description: "List issues with optional filters and pagination",
        destructive: false,
        returns: "IssueList",
        params: &[
            ParamSpec {
                name: "take",
                ty: "integer",
                required: false,
                description: "Number of issues to return",
            },
            ParamSpec {
                name: "skip",
                ty: "integer",
                required: false,
                description: "Number of issues to skip",
            },
            ParamSpec {
                name: "filter",
                ty: "string",
                required: false,
                description: "Filter by status: open|resolved",
            },
            ParamSpec {
                name: "sort",
                ty: "string",
                required: false,
                description: "Sort order: added|modified",
            },
        ],
    },
    ActionSpec {
        name: "issue.get",
        description: "Get a single issue by ID",
        destructive: false,
        returns: "Issue",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Issue ID",
        }],
    },
    ActionSpec {
        name: "issue.create",
        description: "Create a new issue report for a media item",
        destructive: false,
        returns: "Issue",
        params: &[
            ParamSpec {
                name: "issue_type",
                ty: "integer",
                required: true,
                description: "Issue type: 1=video, 2=audio, 3=subtitle, 4=other",
            },
            ParamSpec {
                name: "message",
                ty: "string",
                required: true,
                description: "Description of the issue",
            },
            ParamSpec {
                name: "media_id",
                ty: "integer",
                required: true,
                description: "Overseerr media ID (not TMDB ID)",
            },
            ParamSpec {
                name: "problem_season",
                ty: "integer",
                required: false,
                description: "Season number with the issue (TV only)",
            },
            ParamSpec {
                name: "problem_episode",
                ty: "integer",
                required: false,
                description: "Episode number with the issue (TV only)",
            },
        ],
    },
    ActionSpec {
        name: "issue.comment",
        description: "Add a comment to an existing issue",
        destructive: false,
        returns: "IssueComment",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Issue ID",
            },
            ParamSpec {
                name: "message",
                ty: "string",
                required: true,
                description: "Comment text",
            },
        ],
    },
    // ── Requests (extended) ───────────────────────────────────────────────────
    ActionSpec {
        name: "request.retry",
        description: "Retry a failed media request",
        destructive: false,
        returns: "MediaRequest",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Request ID to retry",
        }],
    },
    ActionSpec {
        name: "request.count",
        description: "Get request counts broken down by status",
        destructive: false,
        returns: "RequestCount",
        params: &[],
    },
    // ── Issues (extended) ────────────────────────────────────────────────────
    ActionSpec {
        name: "issue.update",
        description: "Update the status of an issue (resolved or open)",
        destructive: false,
        returns: "Issue",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Issue ID",
            },
            ParamSpec {
                name: "status",
                ty: "string",
                required: true,
                description: "New status: resolved|open",
            },
        ],
    },
    // ── Media ────────────────────────────────────────────────────────────────
    ActionSpec {
        name: "media.delete",
        description: "Delete a media item permanently by Overseerr media ID",
        destructive: true,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "Overseerr media ID",
        }],
    },
    ActionSpec {
        name: "media.update-status",
        description: "Update the status of a media item",
        destructive: false,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "Overseerr media ID",
            },
            ParamSpec {
                name: "status",
                ty: "string",
                required: true,
                description: "New status (e.g. available, unknown, blacklisted)",
            },
        ],
    },
    // ── Users (extended) ─────────────────────────────────────────────────────
    ActionSpec {
        name: "user.requests",
        description: "List media requests belonging to a user",
        destructive: false,
        returns: "RequestList",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "User ID",
        }],
    },
    ActionSpec {
        name: "user.quota",
        description: "Get the quota information for a user",
        destructive: false,
        returns: "UserQuota",
        params: &[ParamSpec {
            name: "id",
            ty: "integer",
            required: true,
            description: "User ID",
        }],
    },
    ActionSpec {
        name: "user.edit",
        description: "Update a user by ID with a JSON body of fields to change",
        destructive: false,
        returns: "User",
        params: &[
            ParamSpec {
                name: "id",
                ty: "integer",
                required: true,
                description: "User ID",
            },
            ParamSpec {
                name: "body",
                ty: "object",
                required: true,
                description: "Partial user fields to update (JSON object)",
            },
        ],
    },
    // ── Settings / Jobs ───────────────────────────────────────────────────────
    ActionSpec {
        name: "job.run",
        description: "Trigger a background job by its job ID string",
        destructive: false,
        returns: "JobResult",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Job ID (e.g. plex-scan, radarr-scan)",
        }],
    },
    // ── Discover (extended) ───────────────────────────────────────────────────
    ActionSpec {
        name: "discover.trending",
        description: "Get trending media from the Overseerr discover endpoint",
        destructive: false,
        returns: "DiscoverResponse",
        params: &[],
    },
];
