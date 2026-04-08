//! Top-level CLI — clap derive definitions and dispatch router.
//!
//! Every subcommand is a thin shim that parses args, calls into a
//! `lab-apis` client (or a lab-local subsystem), and formats output.
//! See `crates/lab/src/cli/CLAUDE.md` for the rulebook.

pub mod completions;
pub mod doctor;
pub mod health;
pub mod help;
pub mod install;
pub mod plugins;
pub mod serve;

#[cfg(feature = "radarr")]
pub mod radarr;
#[cfg(feature = "sonarr")]
pub mod sonarr;
#[cfg(feature = "prowlarr")]
pub mod prowlarr;
#[cfg(feature = "plex")]
pub mod plex;
#[cfg(feature = "tautulli")]
pub mod tautulli;
#[cfg(feature = "sabnzbd")]
pub mod sabnzbd;
#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;
#[cfg(feature = "tailscale")]
pub mod tailscale;
#[cfg(feature = "linkding")]
pub mod linkding;
#[cfg(feature = "memos")]
pub mod memos;
#[cfg(feature = "bytestash")]
pub mod bytestash;
#[cfg(feature = "paperless")]
pub mod paperless;
#[cfg(feature = "arcane")]
pub mod arcane;
#[cfg(feature = "unraid")]
pub mod unraid;
#[cfg(feature = "unifi")]
pub mod unifi;
#[cfg(feature = "overseerr")]
pub mod overseerr;
#[cfg(feature = "gotify")]
pub mod gotify;
#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "qdrant")]
pub mod qdrant;
#[cfg(feature = "tei")]
pub mod tei;
#[cfg(feature = "apprise")]
pub mod apprise;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

/// `lab` — pluggable homelab CLI + MCP server SDK.
#[derive(Debug, Parser)]
#[command(name = "lab", version, about, long_about = None, disable_help_subcommand = true)]
pub struct Cli {
    /// Emit JSON instead of human-readable tables.
    #[arg(long, global = true)]
    pub json: bool,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Resolved output format based on the `--json` flag.
    #[must_use]
    pub const fn format(&self) -> OutputFormat {
        OutputFormat::from_json_flag(self.json)
    }
}

/// Every top-level subcommand. Service subcommands are added in later
/// plans as each service comes online.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the MCP server (stdio or HTTP transport).
    Serve(serve::ServeArgs),
    /// Audit configured services and report problems.
    Doctor,
    /// Quick reachability check for configured services.
    Health,
    /// Open the plugin manager TUI.
    Plugins,
    /// Install one or more services into `.mcp.json`.
    Install(install::InstallArgs),
    /// Uninstall services from `.mcp.json`.
    Uninstall(install::UninstallArgs),
    /// First-time setup wizard.
    Init,
    /// Print the service + action catalog.
    Help,
    /// Generate shell completions.
    Completions(completions::CompletionsArgs),
    /// Radarr movie collection manager.
    #[cfg(feature = "radarr")]
    Radarr(radarr::RadarrArgs),
    /// Sonarr TV series manager.
    #[cfg(feature = "sonarr")]
    Sonarr(sonarr::SonarrArgs),
    /// Prowlarr indexer manager.
    #[cfg(feature = "prowlarr")]
    Prowlarr(prowlarr::ProwlarrArgs),
    /// Plex media server.
    #[cfg(feature = "plex")]
    Plex(plex::PlexArgs),
    /// Tautulli Plex analytics.
    #[cfg(feature = "tautulli")]
    Tautulli(tautulli::TautulliArgs),
    /// `SABnzbd` download client.
    #[cfg(feature = "sabnzbd")]
    Sabnzbd(sabnzbd::SabnzbdArgs),
    /// qBittorrent download client.
    #[cfg(feature = "qbittorrent")]
    Qbittorrent(qbittorrent::QbittorrentArgs),
    /// Tailscale VPN network.
    #[cfg(feature = "tailscale")]
    Tailscale(tailscale::TailscaleArgs),
    /// Linkding bookmark manager.
    #[cfg(feature = "linkding")]
    Linkding(linkding::LinkdingArgs),
    /// Memos note-taking service.
    #[cfg(feature = "memos")]
    Memos(memos::MemosArgs),
    /// Bytestash snippet manager.
    #[cfg(feature = "bytestash")]
    Bytestash(bytestash::BytestashArgs),
    /// Paperless-ngx document manager.
    #[cfg(feature = "paperless")]
    Paperless(paperless::PaperlessArgs),
    /// Arcane Docker management UI.
    #[cfg(feature = "arcane")]
    Arcane(arcane::ArcaneArgs),
    /// Unraid server management.
    #[cfg(feature = "unraid")]
    Unraid(unraid::UnraidArgs),
    /// `UniFi` network management.
    #[cfg(feature = "unifi")]
    Unifi(unifi::UnifiArgs),
    /// Overseerr media request manager.
    #[cfg(feature = "overseerr")]
    Overseerr(overseerr::OverseerrArgs),
    /// Gotify push notifications.
    #[cfg(feature = "gotify")]
    Gotify(gotify::GotifyArgs),
    /// `OpenAI` API client.
    #[cfg(feature = "openai")]
    Openai(openai::OpenaiArgs),
    /// Qdrant vector database.
    #[cfg(feature = "qdrant")]
    Qdrant(qdrant::QdrantArgs),
    /// HF Text Embeddings Inference.
    #[cfg(feature = "tei")]
    Tei(tei::TeiArgs),
    /// Apprise notification dispatcher.
    #[cfg(feature = "apprise")]
    Apprise(apprise::AppriseArgs),
}

/// Dispatch a parsed [`Cli`] to the correct handler.
pub async fn dispatch(cli: Cli) -> Result<ExitCode> {
    let format = cli.format();
    match cli.command {
        Command::Serve(args) => serve::run(args).await,
        Command::Doctor => doctor::run(format).await,
        Command::Health => health::run(format).await,
        Command::Plugins => Ok(plugins::run()),
        Command::Install(args) => Ok(install::run_install(&args)),
        Command::Uninstall(args) => Ok(install::run_uninstall(&args)),
        Command::Init => Ok(install::run_init()),
        Command::Help => help::run(format),
        Command::Completions(args) => Ok(completions::run(&args)),
        #[cfg(feature = "radarr")]
        Command::Radarr(args) => radarr::run(args, format).await,
        #[cfg(feature = "sonarr")]
        Command::Sonarr(args) => sonarr::run(args, format).await,
        #[cfg(feature = "prowlarr")]
        Command::Prowlarr(args) => prowlarr::run(args, format).await,
        #[cfg(feature = "plex")]
        Command::Plex(args) => plex::run(args, format).await,
        #[cfg(feature = "tautulli")]
        Command::Tautulli(args) => tautulli::run(args, format).await,
        #[cfg(feature = "sabnzbd")]
        Command::Sabnzbd(args) => sabnzbd::run(args, format).await,
        #[cfg(feature = "qbittorrent")]
        Command::Qbittorrent(args) => qbittorrent::run(args, format).await,
        #[cfg(feature = "tailscale")]
        Command::Tailscale(args) => tailscale::run(args, format).await,
        #[cfg(feature = "linkding")]
        Command::Linkding(args) => linkding::run(args, format).await,
        #[cfg(feature = "memos")]
        Command::Memos(args) => memos::run(args, format).await,
        #[cfg(feature = "bytestash")]
        Command::Bytestash(args) => bytestash::run(args, format).await,
        #[cfg(feature = "paperless")]
        Command::Paperless(args) => paperless::run(args, format).await,
        #[cfg(feature = "arcane")]
        Command::Arcane(args) => arcane::run(args, format).await,
        #[cfg(feature = "unraid")]
        Command::Unraid(args) => unraid::run(args, format).await,
        #[cfg(feature = "unifi")]
        Command::Unifi(args) => unifi::run(args, format).await,
        #[cfg(feature = "overseerr")]
        Command::Overseerr(args) => overseerr::run(args, format).await,
        #[cfg(feature = "gotify")]
        Command::Gotify(args) => gotify::run(args, format).await,
        #[cfg(feature = "openai")]
        Command::Openai(args) => openai::run(args, format).await,
        #[cfg(feature = "qdrant")]
        Command::Qdrant(args) => qdrant::run(args, format).await,
        #[cfg(feature = "tei")]
        Command::Tei(args) => tei::run(args, format).await,
        #[cfg(feature = "apprise")]
        Command::Apprise(args) => apprise::run(args, format).await,
    }
}
