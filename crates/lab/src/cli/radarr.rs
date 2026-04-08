//! `lab radarr` — CLI shim for the Radarr service.
//!
//! Thin shim: parse → call client → format. No business logic here.
//! See `crates/lab/src/cli/CLAUDE.md` for the shim rules.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use lab_apis::radarr::types::download_client::DownloadClientId;
use lab_apis::radarr::types::movie::TmdbId;
use lab_apis::radarr::types::notification::NotificationId;
use lab_apis::radarr::types::{CommandId, IndexerId, Movie, MovieId, QueueItemId};

use crate::output::{OutputFormat, print};

/// `lab radarr` arguments.
#[derive(Debug, Args)]
pub struct RadarrArgs {
    #[command(subcommand)]
    pub command: RadarrCommand,
}

/// Radarr subcommands.
#[derive(Debug, Subcommand)]
pub enum RadarrCommand {
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
    let client = crate::mcp::services::radarr::client_from_env()
        .ok_or_else(|| anyhow::anyhow!("RADARR_URL and RADARR_API_KEY must be set"))?;

    match args.command {
        // ── system ───────────────────────────────────────────────────────
        RadarrCommand::SystemStatus => {
            print(&client.system_status().await?, format)?;
        }
        RadarrCommand::SystemHealth => {
            print(&client.health_checks().await?, format)?;
        }
        RadarrCommand::SystemDiskSpace => {
            print(&client.disk_space().await?, format)?;
        }
        RadarrCommand::SystemLogs => {
            print(&client.log_files().await?, format)?;
        }
        RadarrCommand::SystemUpdates => {
            print(&client.updates().await?, format)?;
        }

        // ── movies ───────────────────────────────────────────────────────
        RadarrCommand::MovieList => {
            print(&client.movie_list().await?, format)?;
        }
        RadarrCommand::MovieGet { id } => {
            print(&client.movie_get(MovieId(id)).await?, format)?;
        }
        RadarrCommand::MovieLookup { query } => {
            print(&client.movie_lookup(&query).await?, format)?;
        }
        RadarrCommand::MovieAdd {
            tmdb_id,
            title,
            quality_profile_id,
            root_folder_path,
            monitored,
        } => {
            let movie = Movie {
                id: MovieId(0),
                title,
                year: 0,
                tmdb_id: TmdbId(tmdb_id),
                imdb_id: None,
                has_file: false,
                monitored,
                quality_profile_id: Some(quality_profile_id),
                root_folder_path: Some(root_folder_path),
                path: None,
                size_on_disk: 0,
            };
            print(&client.movie_add(&movie).await?, format)?;
        }
        RadarrCommand::MovieDelete {
            id,
            delete_files,
            yes,
        } => {
            confirm_destructive(yes, &format!("delete movie {id}"))?;
            client.movie_delete(MovieId(id), delete_files).await?;
            tracing::info!("Deleted movie {id}");
        }

        // ── queue ────────────────────────────────────────────────────────
        RadarrCommand::QueueList => {
            print(&client.queue_list().await?, format)?;
        }
        RadarrCommand::QueueRemove {
            id,
            remove_from_client,
            blocklist,
            yes,
        } => {
            confirm_destructive(yes, &format!("remove queue item {id}"))?;
            client
                .queue_remove(QueueItemId(id), remove_from_client, blocklist)
                .await?;
            tracing::info!("Removed queue item {id}");
        }

        // ── calendar ─────────────────────────────────────────────────────
        RadarrCommand::CalendarList { start, end } => {
            print(
                &client
                    .calendar_list(start.as_deref(), end.as_deref())
                    .await?,
                format,
            )?;
        }

        // ── commands ──────────────────────────────────────────────────────
        RadarrCommand::CommandRefresh { movie_id } => {
            print(
                &client.command_refresh_movie(movie_id.map(MovieId)).await?,
                format,
            )?;
        }
        RadarrCommand::CommandSearch { movie_ids } => {
            let ids: Vec<MovieId> = movie_ids.into_iter().map(MovieId).collect();
            print(&client.command_movies_search(&ids).await?, format)?;
        }
        RadarrCommand::CommandGet { id } => {
            print(&client.command_get(CommandId(id)).await?, format)?;
        }

        // ── history ──────────────────────────────────────────────────────
        RadarrCommand::HistoryList { page, page_size } => {
            print(&client.history_list(page, page_size).await?, format)?;
        }
        RadarrCommand::BlocklistList => {
            print(&client.blocklist_list().await?, format)?;
        }

        // ── releases ─────────────────────────────────────────────────────
        RadarrCommand::ReleaseSearch { movie_id } => {
            print(&client.release_search(MovieId(movie_id)).await?, format)?;
        }

        // ── indexers ─────────────────────────────────────────────────────
        RadarrCommand::IndexerList => {
            print(&client.indexer_list().await?, format)?;
        }
        RadarrCommand::IndexerTest { id } => {
            client.indexer_test(IndexerId(id)).await?;
            tracing::info!("Indexer {id} OK");
        }

        // ── quality ──────────────────────────────────────────────────────
        RadarrCommand::QualityProfileList => {
            print(&client.quality_profile_list().await?, format)?;
        }
        RadarrCommand::QualityDefinitionList => {
            print(&client.quality_definition_list().await?, format)?;
        }

        // ── root folders ──────────────────────────────────────────────────
        RadarrCommand::RootFolderList => {
            print(&client.root_folder_list().await?, format)?;
        }

        // ── tags ─────────────────────────────────────────────────────────
        RadarrCommand::TagList => {
            print(&client.tag_list().await?, format)?;
        }
        RadarrCommand::TagDetailList => {
            print(&client.tag_detail_list().await?, format)?;
        }

        // ── download clients ──────────────────────────────────────────────
        RadarrCommand::DownloadClientList => {
            print(&client.download_client_list().await?, format)?;
        }
        RadarrCommand::DownloadClientTest { id } => {
            client.download_client_test(DownloadClientId(id)).await?;
            tracing::info!("Download client {id} OK");
        }
        RadarrCommand::RemotePathMappingList => {
            print(&client.remote_path_mapping_list().await?, format)?;
        }

        // ── config ────────────────────────────────────────────────────────
        RadarrCommand::ConfigHost => {
            print(&client.host_config_get().await?, format)?;
        }
        RadarrCommand::ConfigNaming => {
            print(&client.naming_config_get().await?, format)?;
        }
        RadarrCommand::ConfigUi => {
            print(&client.ui_config_get().await?, format)?;
        }

        // ── notifications ─────────────────────────────────────────────────
        RadarrCommand::NotificationList => {
            print(&client.notification_list().await?, format)?;
        }
        RadarrCommand::NotificationTest { id } => {
            client.notification_test(NotificationId(id)).await?;
            tracing::info!("Notification {id} OK");
        }

        // ── import lists ──────────────────────────────────────────────────
        RadarrCommand::ImportListList => {
            print(&client.import_list_list().await?, format)?;
        }
        RadarrCommand::ImportListExclusionList => {
            print(&client.import_list_exclusion_list().await?, format)?;
        }

        // ── language ──────────────────────────────────────────────────────
        RadarrCommand::LanguageList => {
            print(&client.language_list().await?, format)?;
        }

        // ── metadata ──────────────────────────────────────────────────────
        RadarrCommand::MetadataList => {
            print(&client.metadata_list().await?, format)?;
        }

        // ── filesystem ────────────────────────────────────────────────────
        RadarrCommand::FilesystemList { path } => {
            print(&client.filesystem_list(&path).await?, format)?;
        }
    }

    Ok(ExitCode::SUCCESS)
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
