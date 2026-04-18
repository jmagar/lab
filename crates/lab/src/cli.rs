//! Top-level CLI — clap derive definitions and dispatch router.
//!
//! Every subcommand is a thin shim that parses args, calls into a
//! `lab-apis` client (or a lab-local subsystem), and formats output.
//! See `crates/lab/src/cli/CLAUDE.md` for the rulebook.

pub mod audit;
pub mod completions;
pub mod device;
pub mod doctor;
pub mod extract;
pub mod gateway;
pub mod health;
pub mod help;
pub mod helpers;
pub mod install;
pub mod logs;
pub mod oauth;
pub mod params;
pub mod plugins;
pub mod scaffold;
pub mod serve;

#[cfg(feature = "apprise")]
pub mod apprise;
#[cfg(feature = "arcane")]
pub mod arcane;
#[cfg(feature = "bytestash")]
pub mod bytestash;
#[cfg(feature = "deploy")]
pub mod deploy;
#[cfg(feature = "gotify")]
pub mod gotify;
#[cfg(feature = "linkding")]
pub mod linkding;
#[cfg(feature = "memos")]
pub mod memos;
#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "overseerr")]
pub mod overseerr;
#[cfg(feature = "paperless")]
pub mod paperless;
#[cfg(feature = "plex")]
pub mod plex;
#[cfg(feature = "prowlarr")]
pub mod prowlarr;
#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;
#[cfg(feature = "qdrant")]
pub mod qdrant;
#[cfg(feature = "radarr")]
pub mod radarr;
#[cfg(feature = "sabnzbd")]
pub mod sabnzbd;
#[cfg(feature = "sonarr")]
pub mod sonarr;
#[cfg(feature = "tailscale")]
pub mod tailscale;
#[cfg(feature = "tautulli")]
pub mod tautulli;
#[cfg(feature = "tei")]
pub mod tei;
#[cfg(feature = "unifi")]
pub mod unifi;
#[cfg(feature = "unraid")]
pub mod unraid;

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::LabConfig;
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
    /// Query fleet devices from the configured master.
    Device(device::DeviceArgs),
    /// Quick reachability check for configured services.
    Health,
    /// Open the plugin manager TUI.
    Plugins,
    /// Audit service onboarding against the repo contract.
    Audit(audit::AuditArgs),
    /// Install one or more services into `.mcp.json`.
    Install(install::InstallArgs),
    /// Uninstall services from `.mcp.json`.
    Uninstall(install::UninstallArgs),
    /// First-time setup wizard.
    Init,
    /// Print the service + action catalog.
    Help,
    /// Generate a new service onboarding scaffold.
    Scaffold(scaffold::ScaffoldArgs),
    /// Generate shell completions.
    Completions(completions::CompletionsArgs),
    /// Scan a local or SSH appdata path and extract service credentials.
    Extract(extract::ExtractCmd),
    /// Manage proxied upstream MCP gateways.
    Gateway(gateway::GatewayArgs),
    /// Run local OAuth callback relay helpers.
    Oauth(oauth::OauthArgs),
    /// Search fleet logs on the configured master.
    Logs(logs::LogsArgs),
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
    /// Deploy the local lab release binary to SSH targets.
    #[cfg(feature = "deploy")]
    Deploy(deploy::DeployArgs),
}

/// Dispatch a parsed [`Cli`] to the correct handler.
pub async fn dispatch(cli: Cli, config: LabConfig) -> Result<ExitCode> {
    let format = cli.format();
    match cli.command {
        Command::Serve(args) => serve::run(args, &config).await,
        Command::Doctor => doctor::run(format),
        Command::Device(args) => device::run(args, format, &config).await,
        Command::Health => health::run(format).await,
        Command::Plugins => plugins::run(),
        Command::Audit(args) => audit::run(args, format),
        Command::Install(args) => install::run_install(&args).map(|()| ExitCode::SUCCESS),
        Command::Uninstall(args) => install::run_uninstall(&args).map(|()| ExitCode::SUCCESS),
        Command::Init => install::run_init().map(|()| ExitCode::SUCCESS),
        Command::Help => help::run(format),
        Command::Scaffold(args) => scaffold::run(args, format),
        Command::Completions(args) => completions::run(&args),
        Command::Extract(cmd) => cmd.run().await.map(|()| ExitCode::SUCCESS),
        Command::Gateway(args) => gateway::run(args, format, &config).await,
        Command::Oauth(args) => oauth::run(args, &config).await,
        Command::Logs(args) => logs::run(args, format, &config).await,
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
        #[cfg(feature = "deploy")]
        Command::Deploy(args) => {
            let deploy_prefs = config.deploy.clone().unwrap_or_default();
            let runner =
                crate::dispatch::deploy::runner::build_default_runner(deploy_prefs);
            deploy::run(args, format, &runner)
                .await
                .map(|()| ExitCode::SUCCESS)
        }
    }
}
