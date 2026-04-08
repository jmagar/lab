//! MCP dispatch for the Radarr tool.

use serde_json::Value;

use lab_apis::core::Auth;
use lab_apis::core::action::{ActionSpec, ParamSpec};
use lab_apis::radarr::RadarrClient;
use lab_apis::radarr::error::RadarrError;
use lab_apis::radarr::types::download_client::DownloadClientId;
use lab_apis::radarr::types::movie::TmdbId;
use lab_apis::radarr::types::notification::NotificationId;
use lab_apis::radarr::types::{CommandId, IndexerId, Movie, MovieId, QueueItemId};

use crate::mcp::envelope::ToolError;

impl From<RadarrError> for ToolError {
    fn from(e: RadarrError) -> Self {
        let kind = match &e {
            RadarrError::Api(api) => api.kind(),
            RadarrError::NotFound { .. } => "not_found",
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Action catalog for the radarr tool.
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "system.status",
        description: "Return Radarr system status and version",
        destructive: false,
        returns: "SystemStatus",
        params: &[],
    },
    ActionSpec {
        name: "system.health",
        description: "Return Radarr health check results",
        destructive: false,
        returns: "HealthCheck[]",
        params: &[],
    },
    ActionSpec {
        name: "system.disk-space",
        description: "Return disk space information for all drives",
        destructive: false,
        returns: "DiskSpace[]",
        params: &[],
    },
    ActionSpec {
        name: "system.logs",
        description: "Return list of available log files",
        destructive: false,
        returns: "LogFile[]",
        params: &[],
    },
    ActionSpec {
        name: "system.updates",
        description: "Return available Radarr updates",
        destructive: false,
        returns: "UpdateInfo[]",
        params: &[],
    },
    ActionSpec {
        name: "movie.list",
        description: "List all movies in the Radarr library",
        destructive: false,
        returns: "Movie[]",
        params: &[],
    },
    ActionSpec {
        name: "movie.get",
        description: "Get a single movie by its Radarr ID",
        destructive: false,
        returns: "Movie",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Radarr movie ID",
        }],
    },
    ActionSpec {
        name: "movie.lookup",
        description: "Search for movies to add (TMDB / IMDB lookup)",
        destructive: false,
        returns: "MovieLookup[]",
        params: &[ParamSpec {
            name: "query",
            ty: "string",
            required: true,
            description: "Search term, TMDB ID (tmdb:12345), or IMDB ID (imdb:tt1234567)",
        }],
    },
    ActionSpec {
        name: "movie.add",
        description: "Add a movie to Radarr for monitoring and download",
        destructive: false,
        returns: "Movie",
        params: &[
            ParamSpec {
                name: "tmdb_id",
                ty: "i64",
                required: true,
                description: "TMDB ID of the movie",
            },
            ParamSpec {
                name: "title",
                ty: "string",
                required: true,
                description: "Movie title",
            },
            ParamSpec {
                name: "quality_profile_id",
                ty: "i64",
                required: true,
                description: "Quality profile ID (get from quality-profile.list)",
            },
            ParamSpec {
                name: "root_folder_path",
                ty: "string",
                required: true,
                description: "Root folder path (get from root-folder.list)",
            },
            ParamSpec {
                name: "monitored",
                ty: "bool",
                required: false,
                description: "Monitor movie for download (default true)",
            },
            ParamSpec {
                name: "year",
                ty: "i32",
                required: false,
                description: "Release year (default 0)",
            },
        ],
    },
    ActionSpec {
        name: "movie.delete",
        description: "Delete a movie from Radarr",
        destructive: true,
        returns: "void",
        params: &[
            ParamSpec {
                name: "id",
                ty: "i64",
                required: true,
                description: "Radarr movie ID",
            },
            ParamSpec {
                name: "delete_files",
                ty: "bool",
                required: false,
                description: "Also delete files from disk (default false)",
            },
        ],
    },
    ActionSpec {
        name: "queue.list",
        description: "List all items currently in the download queue",
        destructive: false,
        returns: "QueueItem[]",
        params: &[],
    },
    ActionSpec {
        name: "queue.remove",
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
                name: "remove_from_client",
                ty: "bool",
                required: false,
                description: "Remove from download client too (default true)",
            },
            ParamSpec {
                name: "blocklist",
                ty: "bool",
                required: false,
                description: "Add release to blocklist (default false)",
            },
        ],
    },
    ActionSpec {
        name: "calendar.list",
        description: "List upcoming movie releases",
        destructive: false,
        returns: "CalendarEntry[]",
        params: &[
            ParamSpec {
                name: "start",
                ty: "string",
                required: false,
                description: "Start date ISO 8601 (default today)",
            },
            ParamSpec {
                name: "end",
                ty: "string",
                required: false,
                description: "End date ISO 8601 (default 7 days from now)",
            },
        ],
    },
    ActionSpec {
        name: "command.refresh",
        description: "Refresh metadata for one movie or all movies",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "movie_id",
            ty: "i64",
            required: false,
            description: "Movie ID to refresh (omit to refresh all)",
        }],
    },
    ActionSpec {
        name: "command.search",
        description: "Trigger a file search for specified movies",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "movie_ids",
            ty: "i64[]",
            required: true,
            description: "Array of Radarr movie IDs to search",
        }],
    },
    ActionSpec {
        name: "command.get",
        description: "Get the status of a previously issued command",
        destructive: false,
        returns: "Command",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Command ID",
        }],
    },
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
                description: "Page number (default 1)",
            },
            ParamSpec {
                name: "page_size",
                ty: "u32",
                required: false,
                description: "Items per page (default 10)",
            },
        ],
    },
    ActionSpec {
        name: "blocklist.list",
        description: "List blocked releases",
        destructive: false,
        returns: "BlocklistItem[]",
        params: &[],
    },
    ActionSpec {
        name: "release.search",
        description: "Search indexers for available releases for a movie",
        destructive: false,
        returns: "Release[]",
        params: &[ParamSpec {
            name: "movie_id",
            ty: "i64",
            required: true,
            description: "Radarr movie ID",
        }],
    },
    ActionSpec {
        name: "indexer.list",
        description: "List configured indexers",
        destructive: false,
        returns: "Indexer[]",
        params: &[],
    },
    ActionSpec {
        name: "indexer.test",
        description: "Test an indexer connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Indexer ID",
        }],
    },
    ActionSpec {
        name: "quality-profile.list",
        description: "List quality profiles",
        destructive: false,
        returns: "QualityProfile[]",
        params: &[],
    },
    ActionSpec {
        name: "quality-definition.list",
        description: "List quality definitions",
        destructive: false,
        returns: "QualityDefinition[]",
        params: &[],
    },
    ActionSpec {
        name: "root-folder.list",
        description: "List root folders",
        destructive: false,
        returns: "RootFolder[]",
        params: &[],
    },
    ActionSpec {
        name: "tag.list",
        description: "List all tags",
        destructive: false,
        returns: "Tag[]",
        params: &[],
    },
    ActionSpec {
        name: "tag.detail-list",
        description: "List tags with details (linked movies, etc.)",
        destructive: false,
        returns: "TagDetail[]",
        params: &[],
    },
    ActionSpec {
        name: "download-client.list",
        description: "List configured download clients",
        destructive: false,
        returns: "DownloadClient[]",
        params: &[],
    },
    ActionSpec {
        name: "download-client.test",
        description: "Test a download client connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Download client ID",
        }],
    },
    ActionSpec {
        name: "remote-path-mapping.list",
        description: "List remote path mappings",
        destructive: false,
        returns: "RemotePathMapping[]",
        params: &[],
    },
    ActionSpec {
        name: "config.host",
        description: "Get host configuration",
        destructive: false,
        returns: "HostConfig",
        params: &[],
    },
    ActionSpec {
        name: "config.naming",
        description: "Get file naming configuration",
        destructive: false,
        returns: "NamingConfig",
        params: &[],
    },
    ActionSpec {
        name: "config.ui",
        description: "Get UI configuration",
        destructive: false,
        returns: "UiConfig",
        params: &[],
    },
    ActionSpec {
        name: "notification.list",
        description: "List configured notifications",
        destructive: false,
        returns: "Notification[]",
        params: &[],
    },
    ActionSpec {
        name: "notification.test",
        description: "Test a notification connection",
        destructive: false,
        returns: "void",
        params: &[ParamSpec {
            name: "id",
            ty: "i64",
            required: true,
            description: "Notification ID",
        }],
    },
    ActionSpec {
        name: "import-list.list",
        description: "List configured import lists",
        destructive: false,
        returns: "ImportList[]",
        params: &[],
    },
    ActionSpec {
        name: "import-list.exclusion-list",
        description: "List import list exclusions",
        destructive: false,
        returns: "ImportListExclusion[]",
        params: &[],
    },
    ActionSpec {
        name: "language.list",
        description: "List available languages",
        destructive: false,
        returns: "Language[]",
        params: &[],
    },
    ActionSpec {
        name: "metadata.list",
        description: "List metadata providers",
        destructive: false,
        returns: "Metadata[]",
        params: &[],
    },
    ActionSpec {
        name: "filesystem.list",
        description: "Browse the server filesystem",
        destructive: false,
        returns: "FilesystemListing",
        params: &[ParamSpec {
            name: "path",
            ty: "string",
            required: true,
            description: "Directory path to browse",
        }],
    },
];

/// Build a Radarr client from the default-instance env vars.
#[must_use]
pub fn client_from_env() -> Option<RadarrClient> {
    let url = std::env::var("RADARR_URL").ok()?;
    let key = std::env::var("RADARR_API_KEY").ok()?;
    Some(RadarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    ))
}

fn require_client() -> Result<RadarrClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "RADARR_URL or RADARR_API_KEY not configured".to_string(),
    })
}

fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Dispatch one MCP call against the Radarr tool.
///
/// # Errors
#[allow(clippy::too_many_lines)]
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "radarr",
            "actions": ACTIONS.iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "returns": a.returns,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),

        // ── system ──────────────────────────────────────────────────────────
        "system.status" => {
            let status = require_client()?.system_status().await?;
            to_json(status)
        }
        "system.health" => {
            let checks = require_client()?.health_checks().await?;
            to_json(checks)
        }
        "system.disk-space" => {
            let disks = require_client()?.disk_space().await?;
            to_json(disks)
        }
        "system.logs" => {
            let logs = require_client()?.log_files().await?;
            to_json(logs)
        }
        "system.updates" => {
            let updates = require_client()?.updates().await?;
            to_json(updates)
        }

        // ── movies ──────────────────────────────────────────────────────────
        "movie.list" => {
            let movies = require_client()?.movie_list().await?;
            to_json(movies)
        }
        "movie.get" => {
            let id = MovieId(require_i64(&params, "id")?);
            let movie = require_client()?.movie_get(id).await?;
            to_json(movie)
        }
        "movie.lookup" => {
            let query = params.get("query").and_then(Value::as_str).ok_or_else(|| {
                ToolError::MissingParam {
                    message: "missing required parameter `query`".to_string(),
                    param: "query".to_string(),
                }
            })?;
            let results = require_client()?.movie_lookup(query).await?;
            to_json(results)
        }
        "movie.add" => {
            let tmdb_id = require_i64(&params, "tmdb_id")?;
            let title = params
                .get("title")
                .and_then(Value::as_str)
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `title`".to_string(),
                    param: "title".to_string(),
                })?
                .to_owned();
            let quality_profile_id = params.get("quality_profile_id").and_then(Value::as_i64);
            let root_folder_path = params
                .get("root_folder_path")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let monitored = params
                .get("monitored")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let year = params
                .get("year")
                .and_then(Value::as_i64)
                .map_or(0, |y| i32::try_from(y).unwrap_or(0));
            let movie = Movie {
                id: MovieId(0),
                title,
                year,
                tmdb_id: TmdbId(tmdb_id),
                imdb_id: None,
                has_file: false,
                monitored,
                quality_profile_id,
                root_folder_path,
                path: None,
                size_on_disk: 0,
            };
            let added = require_client()?.movie_add(&movie).await?;
            to_json(added)
        }
        "movie.delete" => {
            let id = MovieId(require_i64(&params, "id")?);
            let delete_files = params
                .get("delete_files")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            require_client()?.movie_delete(id, delete_files).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }

        // ── queue ───────────────────────────────────────────────────────────
        "queue.list" => {
            let items = require_client()?.queue_list().await?;
            to_json(items)
        }
        "queue.remove" => {
            let id = QueueItemId(require_i64(&params, "id")?);
            let remove_from_client = params
                .get("remove_from_client")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let blocklist = params
                .get("blocklist")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            require_client()?
                .queue_remove(id, remove_from_client, blocklist)
                .await?;
            Ok(serde_json::json!({ "removed": true }))
        }

        // ── calendar ────────────────────────────────────────────────────────
        "calendar.list" => {
            let start = params.get("start").and_then(Value::as_str);
            let end = params.get("end").and_then(Value::as_str);
            let entries = require_client()?.calendar_list(start, end).await?;
            to_json(entries)
        }

        // ── commands ────────────────────────────────────────────────────────
        "command.refresh" => {
            let movie_id = params.get("movie_id").and_then(Value::as_i64).map(MovieId);
            let cmd = require_client()?.command_refresh_movie(movie_id).await?;
            to_json(cmd)
        }
        "command.search" => {
            let ids: Vec<MovieId> = params
                .get("movie_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `movie_ids`".to_string(),
                    param: "movie_ids".to_string(),
                })?
                .iter()
                .filter_map(Value::as_i64)
                .map(MovieId)
                .collect();
            let cmd = require_client()?.command_movies_search(&ids).await?;
            to_json(cmd)
        }
        "command.get" => {
            let id = CommandId(require_i64(&params, "id")?);
            let cmd = require_client()?.command_get(id).await?;
            to_json(cmd)
        }

        // ── history ─────────────────────────────────────────────────────────
        "history.list" => {
            let page = u32::try_from(params.get("page").and_then(Value::as_u64).unwrap_or(1))
                .unwrap_or(u32::MAX);
            let page_size = u32::try_from(
                params
                    .get("page_size")
                    .and_then(Value::as_u64)
                    .unwrap_or(10),
            )
            .unwrap_or(u32::MAX);
            let page_data = require_client()?.history_list(page, page_size).await?;
            to_json(page_data)
        }
        "blocklist.list" => {
            let items = require_client()?.blocklist_list().await?;
            to_json(items)
        }

        // ── releases ────────────────────────────────────────────────────────
        "release.search" => {
            let movie_id = MovieId(require_i64(&params, "movie_id")?);
            let releases = require_client()?.release_search(movie_id).await?;
            to_json(releases)
        }

        // ── indexers ────────────────────────────────────────────────────────
        "indexer.list" => {
            let indexers = require_client()?.indexer_list().await?;
            to_json(indexers)
        }
        "indexer.test" => {
            let id = IndexerId(require_i64(&params, "id")?);
            require_client()?.indexer_test(id).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        // ── quality ─────────────────────────────────────────────────────────
        "quality-profile.list" => {
            let profiles = require_client()?.quality_profile_list().await?;
            to_json(profiles)
        }
        "quality-definition.list" => {
            let defs = require_client()?.quality_definition_list().await?;
            to_json(defs)
        }

        // ── root folders ────────────────────────────────────────────────────
        "root-folder.list" => {
            let folders = require_client()?.root_folder_list().await?;
            to_json(folders)
        }

        // ── tags ────────────────────────────────────────────────────────────
        "tag.list" => {
            let tags = require_client()?.tag_list().await?;
            to_json(tags)
        }
        "tag.detail-list" => {
            let tags = require_client()?.tag_detail_list().await?;
            to_json(tags)
        }

        // ── download clients ────────────────────────────────────────────────
        "download-client.list" => {
            let clients = require_client()?.download_client_list().await?;
            to_json(clients)
        }
        "download-client.test" => {
            let id = DownloadClientId(require_i64(&params, "id")?);
            require_client()?.download_client_test(id).await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "remote-path-mapping.list" => {
            let mappings = require_client()?.remote_path_mapping_list().await?;
            to_json(mappings)
        }

        // ── config ──────────────────────────────────────────────────────────
        "config.host" => {
            let cfg = require_client()?.host_config_get().await?;
            to_json(cfg)
        }
        "config.naming" => {
            let cfg = require_client()?.naming_config_get().await?;
            to_json(cfg)
        }
        "config.ui" => {
            let cfg = require_client()?.ui_config_get().await?;
            to_json(cfg)
        }

        // ── notifications ───────────────────────────────────────────────────
        "notification.list" => {
            let notifications = require_client()?.notification_list().await?;
            to_json(notifications)
        }
        "notification.test" => {
            let id = NotificationId(require_i64(&params, "id")?);
            require_client()?.notification_test(id).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        // ── import lists ────────────────────────────────────────────────────
        "import-list.list" => {
            let lists = require_client()?.import_list_list().await?;
            to_json(lists)
        }
        "import-list.exclusion-list" => {
            let exclusions = require_client()?.import_list_exclusion_list().await?;
            to_json(exclusions)
        }

        // ── language ────────────────────────────────────────────────────────
        "language.list" => {
            let langs = require_client()?.language_list().await?;
            to_json(langs)
        }

        // ── metadata ────────────────────────────────────────────────────────
        "metadata.list" => {
            let meta = require_client()?.metadata_list().await?;
            to_json(meta)
        }

        // ── filesystem ──────────────────────────────────────────────────────
        "filesystem.list" => {
            let path = params.get("path").and_then(Value::as_str).ok_or_else(|| {
                ToolError::MissingParam {
                    message: "missing required parameter `path`".to_string(),
                    param: "path".to_string(),
                }
            })?;
            let listing = require_client()?.filesystem_list(path).await?;
            to_json(listing)
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `radarr`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use lab_apis::core::error::ApiError;
    use lab_apis::radarr::error::RadarrError;

    use super::*;

    // ── From<RadarrError> for ToolError ──────────────────────────────────────

    #[test]
    fn sdk_error_from_api_auth() {
        let e: ToolError = RadarrError::Api(ApiError::Auth).into();
        assert_eq!(e.kind(), "auth_failed");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "auth_failed");
        assert!(v.get("sdk_kind").is_none());
    }

    #[test]
    fn sdk_error_from_api_not_found() {
        let e: ToolError = RadarrError::Api(ApiError::NotFound).into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_radarr_not_found() {
        let e: ToolError = RadarrError::NotFound {
            kind: "movie",
            id: 42,
        }
        .into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_network() {
        let e: ToolError = RadarrError::Api(ApiError::Network("timeout".into())).into();
        assert_eq!(e.kind(), "network_error");
    }

    // ── unknown_action shape ─────────────────────────────────────────────────

    #[test]
    fn unknown_action_includes_valid_list() {
        let e = ToolError::UnknownAction {
            message: "unknown action `bogus`".into(),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "unknown_action");
        let valid = v["valid"].as_array().unwrap();
        assert!(
            valid.iter().any(|s| s == "help"),
            "help must be in valid list"
        );
        assert!(
            valid.iter().any(|s| s == "movie.list"),
            "movie.list must be in valid list"
        );
    }
}
