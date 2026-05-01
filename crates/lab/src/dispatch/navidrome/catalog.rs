use lab_apis::core::action::{ActionSpec, ParamSpec};

/// Action catalog for the `navidrome` service.
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
    ActionSpec {
        name: "server.ping",
        description: "Ping Navidrome through the Subsonic API",
        destructive: false,
        returns: "QueryResponse",
        params: &[],
    },
    ActionSpec {
        name: "artist.list",
        description: "List Navidrome artists",
        destructive: false,
        returns: "QueryResponse",
        params: &[],
    },
    ActionSpec {
        name: "album.list",
        description: "List albums with bounded Subsonic size/offset",
        destructive: false,
        returns: "QueryResponse",
        params: &[
            ParamSpec {
                name: "type",
                ty: "string",
                required: false,
                description: "Subsonic album list type, defaults to newest",
            },
            ParamSpec {
                name: "size",
                ty: "integer",
                required: true,
                description: "Page size, 1-100",
            },
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Page offset, defaults to 0",
            },
        ],
    },
    ActionSpec {
        name: "album.get",
        description: "Fetch one album by string id",
        destructive: false,
        returns: "QueryResponse",
        params: &[ParamSpec {
            name: "id",
            ty: "string",
            required: true,
            description: "Navidrome/Subsonic album id",
        }],
    },
    ActionSpec {
        name: "search.query",
        description: "Search artists, albums, and songs with bounded size/offset",
        destructive: false,
        returns: "QueryResponse",
        params: &[
            ParamSpec {
                name: "query",
                ty: "string",
                required: true,
                description: "Search query",
            },
            ParamSpec {
                name: "size",
                ty: "integer",
                required: true,
                description: "Page size per resource, 1-100",
            },
            ParamSpec {
                name: "offset",
                ty: "integer",
                required: false,
                description: "Page offset, defaults to 0",
            },
        ],
    },
    ActionSpec {
        name: "playlist.list",
        description: "List playlists",
        destructive: false,
        returns: "QueryResponse",
        params: &[],
    },
];
