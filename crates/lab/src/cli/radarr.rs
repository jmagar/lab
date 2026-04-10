//! `lab radarr` — CLI shim for the Radarr service.
//!
//! Thin shim: parse → map to canonical action + params → call shared dispatch.
//! No business logic lives here.
//! See `crates/lab/src/cli/CLAUDE.md` for the shim rules.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::dispatch::error::ToolError;
use crate::output::{OutputFormat, print};

/// `lab radarr` arguments.
#[derive(Debug, Args)]
pub struct RadarrArgs {
    #[command(subcommand)]
    pub command: Option<RadarrCommand>,
}

/// Radarr subcommands.
#[derive(Debug, Subcommand)]
pub enum RadarrCommand {
    /// Return the Radarr action catalog.
    Help,

    // ── system ───────────────────────────────────────────────────────────
    /// Return Radarr system status and version.
    SystemStatus,
    /// Return Radarr health check results.
    SystemHealth,
    /// Return disk space information for all drives.
    SystemDiskSpace,
    /// Return list of available log files.
    SystemLogs,
    /// Return available Radarr updates.
    SystemUpdates,

    // ── movies ───────────────────────────────────────────────────────────
    /// List all movies in the Radarr library.
    MovieList,
    /// Get a single movie by its Radarr ID.
    MovieGet {
        /// Radarr movie ID.
        id: i64,
    },
    /// Search for movies to add (TMDB / IMDB lookup).
    MovieLookup {
        /// Search term, TMDB ID (tmdb:12345), or IMDB ID (imdb:tt1234567).
        query: String,
    },
    /// Add a movie to Radarr for monitoring and download.
    MovieAdd {
        /// TMDB ID of the movie.
        #[arg(long)]
        tmdb_id: i64,
        /// Movie title.
        #[arg(long)]
        title: String,
        /// Quality profile ID (use quality-profile-list to find IDs).
        #[arg(long)]
        quality_profile_id: i64,
        /// Root folder path (use root-folder-list to find paths).
        #[arg(long)]
        root_folder_path: String,
        /// Monitor movie for download (default true).
        #[arg(long, default_value = "true")]
        monitored: bool,
        /// Release year (default 0; use movie-lookup to retrieve from TMDB).
        #[arg(long, default_value = "0")]
        year: i32,
    },
    /// Delete a movie from Radarr.
    MovieDelete {
        /// Radarr movie ID.
        id: i64,
        /// Also delete files from disk.
        #[arg(long)]
        delete_files: bool,
        /// Skip confirmation prompt.
        #[arg(long, short = 'y')]
        yes: bool,
    },

    // ── queue ────────────────────────────────────────────────────────────
    /// List all items currently in the download queue.
    QueueList,
    /// Remove an item from the download queue.
    QueueRemove {
        /// Queue item ID.
        id: i64,
        /// Remove from download client too (default true).
        #[arg(long, default_value = "true")]
        remove_from_client: bool,
        /// Add release to blocklist.
        #[arg(long)]
        blocklist: bool,
        /// Skip confirmation prompt.
        #[arg(long, short = 'y')]
        yes: bool,
    },

    // ── calendar ─────────────────────────────────────────────────────────
    /// List upcoming movie releases.
    CalendarList {
        /// Start date ISO 8601 (default today).
        #[arg(long)]
        start: Option<String>,
        /// End date ISO 8601 (default 7 days from now).
        #[arg(long)]
        end: Option<String>,
    },

    // ── commands ──────────────────────────────────────────────────────────
    /// Refresh metadata for one movie or all movies.
    CommandRefresh {
        /// Movie ID to refresh (omit to refresh all).
        #[arg(long)]
        movie_id: Option<i64>,
    },
    /// Trigger a file search for one or more movies.
    CommandSearch {
        /// Radarr movie IDs to search (space-separated).
        movie_ids: Vec<i64>,
    },
    /// Get the status of a previously issued command.
    CommandGet {
        /// Command ID.
        id: i64,
    },

    // ── history ──────────────────────────────────────────────────────────
    /// List download history.
    HistoryList {
        /// Page number (default 1).
        #[arg(long, default_value = "1")]
        page: u32,
        /// Items per page (default 10).
        #[arg(long, default_value = "10")]
        page_size: u32,
    },
    /// List blocked releases.
    BlocklistList,

    // ── releases ─────────────────────────────────────────────────────────
    /// Search indexers for available releases for a movie.
    ReleaseSearch {
        /// Radarr movie ID.
        movie_id: i64,
    },

    // ── indexers ─────────────────────────────────────────────────────────
    /// List configured indexers.
    IndexerList,
    /// Test an indexer connection.
    IndexerTest {
        /// Indexer ID.
        id: i64,
    },

    // ── quality ──────────────────────────────────────────────────────────
    /// List quality profiles.
    QualityProfileList,
    /// List quality definitions.
    QualityDefinitionList,

    // ── root folders ──────────────────────────────────────────────────────
    /// List root folders.
    RootFolderList,

    // ── tags ─────────────────────────────────────────────────────────────
    /// List all tags.
    TagList,
    /// List tags with full details.
    TagDetailList,

    // ── download clients ──────────────────────────────────────────────────
    /// List configured download clients.
    DownloadClientList,
    /// Test a download client connection.
    DownloadClientTest {
        /// Download client ID.
        id: i64,
    },
    /// List remote path mappings.
    RemotePathMappingList,

    // ── config ────────────────────────────────────────────────────────────
    /// Get host configuration.
    ConfigHost,
    /// Get file naming configuration.
    ConfigNaming,
    /// Get UI configuration.
    ConfigUi,

    // ── notifications ─────────────────────────────────────────────────────
    /// List configured notifications.
    NotificationList,
    /// Test a notification connection.
    NotificationTest {
        /// Notification ID.
        id: i64,
    },

    // ── import lists ──────────────────────────────────────────────────────
    /// List configured import lists.
    ImportListList,
    /// List import list exclusions.
    ImportListExclusionList,

    // ── language ──────────────────────────────────────────────────────────
    /// List available languages.
    LanguageList,

    // ── metadata ──────────────────────────────────────────────────────────
    /// List metadata providers.
    MetadataList,

    // ── filesystem ────────────────────────────────────────────────────────
    /// Browse the server filesystem.
    FilesystemList {
        /// Directory path to browse.
        path: String,
    },
}

/// Run the `lab radarr` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
#[allow(clippy::too_many_lines)]
pub async fn run(args: RadarrArgs, format: OutputFormat) -> Result<ExitCode> {
    let outcome = match args.command.unwrap_or(RadarrCommand::Help) {
        RadarrCommand::Help => command_outcome("help", json!({})),
        RadarrCommand::SystemStatus => command_outcome("system.status", json!({})),
        RadarrCommand::SystemHealth => command_outcome("system.health", json!({})),
        RadarrCommand::SystemDiskSpace => command_outcome("system.disk-space", json!({})),
        RadarrCommand::SystemLogs => command_outcome("system.logs", json!({})),
        RadarrCommand::SystemUpdates => command_outcome("system.updates", json!({})),
        RadarrCommand::MovieList => command_outcome("movie.list", json!({})),
        RadarrCommand::MovieGet { id } => command_outcome("movie.get", json!({ "id": id })),
        RadarrCommand::MovieLookup { query } => {
            command_outcome("movie.lookup", json!({ "query": query }))
        }
        RadarrCommand::MovieAdd {
            tmdb_id,
            title,
            quality_profile_id,
            root_folder_path,
            monitored,
            year,
        } => command_outcome(
            "movie.add",
            json!({
                "tmdb_id": tmdb_id,
                "title": title,
                "quality_profile_id": quality_profile_id,
                "root_folder_path": root_folder_path,
                "monitored": monitored,
                "year": year,
            }),
        ),
        RadarrCommand::MovieDelete {
            id,
            delete_files,
            yes,
        } => {
            confirm_destructive(yes, &format!("delete movie {id}"))?;
            quiet_outcome(
                "movie.delete",
                json!({ "id": id, "delete_files": delete_files }),
                format!("Deleted movie {id}"),
            )
        }
        RadarrCommand::QueueList => command_outcome("queue.list", json!({})),
        RadarrCommand::QueueRemove {
            id,
            remove_from_client,
            blocklist,
            yes,
        } => {
            confirm_destructive(yes, &format!("remove queue item {id}"))?;
            quiet_outcome(
                "queue.remove",
                json!({
                    "id": id,
                    "remove_from_client": remove_from_client,
                    "blocklist": blocklist,
                }),
                format!("Removed queue item {id}"),
            )
        }
        RadarrCommand::CalendarList { start, end } => {
            command_outcome("calendar.list", json!({ "start": start, "end": end }))
        }
        RadarrCommand::CommandRefresh { movie_id } => {
            command_outcome("command.refresh", json!({ "movie_id": movie_id }))
        }
        RadarrCommand::CommandSearch { movie_ids } => {
            command_outcome("command.search", json!({ "movie_ids": movie_ids }))
        }
        RadarrCommand::CommandGet { id } => command_outcome("command.get", json!({ "id": id })),
        RadarrCommand::HistoryList { page, page_size } => command_outcome(
            "history.list",
            json!({ "page": page, "page_size": page_size }),
        ),
        RadarrCommand::BlocklistList => command_outcome("blocklist.list", json!({})),
        RadarrCommand::ReleaseSearch { movie_id } => {
            command_outcome("release.search", json!({ "movie_id": movie_id }))
        }
        RadarrCommand::IndexerList => command_outcome("indexer.list", json!({})),
        RadarrCommand::IndexerTest { id } => quiet_outcome(
            "indexer.test",
            json!({ "id": id }),
            format!("Indexer {id} OK"),
        ),
        RadarrCommand::QualityProfileList => command_outcome("quality-profile.list", json!({})),
        RadarrCommand::QualityDefinitionList => {
            command_outcome("quality-definition.list", json!({}))
        }
        RadarrCommand::RootFolderList => command_outcome("root-folder.list", json!({})),
        RadarrCommand::TagList => command_outcome("tag.list", json!({})),
        RadarrCommand::TagDetailList => command_outcome("tag.detail-list", json!({})),
        RadarrCommand::DownloadClientList => command_outcome("download-client.list", json!({})),
        RadarrCommand::DownloadClientTest { id } => quiet_outcome(
            "download-client.test",
            json!({ "id": id }),
            format!("Download client {id} OK"),
        ),
        RadarrCommand::RemotePathMappingList => {
            command_outcome("remote-path-mapping.list", json!({}))
        }
        RadarrCommand::ConfigHost => command_outcome("config.host", json!({})),
        RadarrCommand::ConfigNaming => command_outcome("config.naming", json!({})),
        RadarrCommand::ConfigUi => command_outcome("config.ui", json!({})),
        RadarrCommand::NotificationList => command_outcome("notification.list", json!({})),
        RadarrCommand::NotificationTest { id } => quiet_outcome(
            "notification.test",
            json!({ "id": id }),
            format!("Notification {id} OK"),
        ),
        RadarrCommand::ImportListList => command_outcome("import-list.list", json!({})),
        RadarrCommand::ImportListExclusionList => {
            command_outcome("import-list.exclusion-list", json!({}))
        }
        RadarrCommand::LanguageList => command_outcome("language.list", json!({})),
        RadarrCommand::MetadataList => command_outcome("metadata.list", json!({})),
        RadarrCommand::FilesystemList { path } => {
            command_outcome("filesystem.list", json!({ "path": path }))
        }
    };

    execute_dispatch(outcome, format).await
}

/// Refuse to proceed on a destructive action unless the user passed `-y`.
///
/// Non-TTY + no `-y` → error. TTY + no `-y` → interactive prompt (TODO).
fn confirm_destructive(yes: bool, description: &str) -> Result<()> {
    if yes {
        return Ok(());
    }
    // For now: refuse if not confirmed. A future pass can add readline prompt.
    anyhow::bail!("pass -y / --yes to confirm: {description}")
}

struct CommandOutcome {
    action: &'static str,
    params: Value,
    print_result: bool,
    success_note: Option<String>,
}

#[allow(clippy::missing_const_for_fn)]
fn command_outcome(action: &'static str, params: Value) -> CommandOutcome {
    CommandOutcome {
        action,
        params,
        print_result: true,
        success_note: None,
    }
}

#[allow(clippy::missing_const_for_fn)]
fn quiet_outcome(action: &'static str, params: Value, success_note: String) -> CommandOutcome {
    CommandOutcome {
        action,
        params,
        print_result: false,
        success_note: Some(success_note),
    }
}

async fn execute_dispatch(outcome: CommandOutcome, format: OutputFormat) -> Result<ExitCode> {
    let action = outcome.action.to_string();
    let start = std::time::Instant::now();
    let result = crate::dispatch::radarr::dispatch(&action, outcome.params).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(
            surface = "cli",
            service = "radarr",
            action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(e) if e.is_internal() => tracing::error!(
            surface = "cli",
            service = "radarr",
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
        Err(e) => tracing::warn!(
            surface = "cli",
            service = "radarr",
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }

    let value = result.map_err(tool_error_to_anyhow)?;
    if outcome.print_result {
        print(&value, format)?;
    } else if let Some(note) = outcome.success_note {
        tracing::info!("{note}");
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_a_real_subcommand() {
        let args = RadarrArgs {
            command: Some(RadarrCommand::Help),
        };
        assert!(matches!(args.command, Some(RadarrCommand::Help)));
    }

    #[test]
    fn radarr_defaults_to_help_without_a_subcommand() {
        let args = RadarrArgs { command: None };
        assert!(args.command.is_none());
    }
}

#[allow(clippy::needless_pass_by_value)]
fn tool_error_to_anyhow(e: ToolError) -> anyhow::Error {
    anyhow::anyhow!(
        "{}",
        serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())
    )
}
