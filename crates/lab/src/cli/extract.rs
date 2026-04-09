//! `lab extract` — CLI surface for the extract module.
//!
//! Thin shim. All logic lives in `lab_apis::extract::ExtractClient`. This
//! file owns:
//!   1. The clap subcommand definition.
//!   2. The destructive-confirmation prompt for `--apply`.
//!   3. The lab-specific `.env` writer (with timestamped backup).
//!   4. The table/JSON output formatter.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;

use lab_apis::extract::{ExtractClient, ExtractReport, Uri};

/// `lab extract <uri> [--apply | --diff] [-y] [--json]`
#[derive(Debug, Args)]
pub struct ExtractCmd {
    /// Appdata path to scan. Local (`/path` or `~/path`) or SSH (`host:/path`).
    pub uri: String,

    /// Write the extracted creds into `~/.lab/.env` (destructive — prompts).
    #[arg(long)]
    pub apply: bool,

    /// Show what would change vs the current `~/.lab/.env`, no writes.
    #[arg(long, conflicts_with = "apply")]
    pub diff: bool,

    /// Skip the destructive-action confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Don't actually write — just show what would happen with `--apply`.
    #[arg(long)]
    pub dry_run: bool,

    /// Render as JSON instead of a table.
    #[arg(long)]
    pub json: bool,

    /// Override the env-file path (defaults to `~/.lab/.env`).
    #[arg(long, value_name = "PATH")]
    pub env_path: Option<PathBuf>,
}

impl ExtractCmd {
    /// Run the command.
    ///
    /// # Errors
    /// Propagates any error from `ExtractClient::scan`, the confirmation
    /// prompt, or the `.env` writer.
    pub async fn run(self) -> Result<()> {
        let uri: Uri = self
            .uri
            .parse()
            .with_context(|| format!("invalid uri: {}", self.uri))?;

        let client = ExtractClient::new();
        let report = client
            .scan(uri)
            .await
            .with_context(|| "scan failed")?;

        if self.apply {
            self.apply_report(&report)?;
        } else if self.diff {
            self.diff_report(&report)?;
        } else {
            self.print_report(&report)?;
        }

        Ok(())
    }

    fn apply_report(&self, report: &ExtractReport) -> Result<()> {
        // 1. Resolve target env file path
        let target = self.resolve_env_path()?;

        // 2. Show what's about to change
        self.print_report(report)?;
        eprintln!("\n→ would write {} fields to {}", report.creds.len(), target.display());

        // 3. Destructive confirmation (unless -y or --dry-run)
        if !self.yes && !self.dry_run && !confirm_destructive("extract.apply")? {
            anyhow::bail!("aborted by user");
        }
        if self.dry_run {
            eprintln!("(dry-run — no changes written)");
            return Ok(());
        }

        // 4. Backup + atomic write — implementation lives in lab/src/config.rs
        //    backup_env(&target)?;
        //    write_env(&target, &report.creds)?;
        anyhow::bail!("apply not yet implemented (would patch {})", target.display())
    }

    fn diff_report(&self, _report: &ExtractReport) -> Result<()> {
        anyhow::bail!("diff not yet implemented")
    }

    fn print_report(&self, report: &ExtractReport) -> Result<()> {
        if self.json {
            // serde_json::to_string_pretty(report)? — printed via output module
            return Ok(());
        }
        // TODO: render via crates/lab/src/output.rs as a tabled::Table
        println!("found {} services under {}", report.found.len(), report.uri.path().display());
        for cred in &report.creds {
            println!(
                "  {:<12} url={:<32} secret={}",
                cred.service,
                cred.url.as_deref().unwrap_or("?"),
                cred.secret.as_deref().map_or("?", |_| "set"),
            );
        }
        for w in &report.warnings {
            println!("  ! {}: {}", w.service, w.message);
        }
        Ok(())
    }

    fn resolve_env_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.env_path {
            return Ok(p.clone());
        }
        let home = std::env::var("HOME").context("$HOME not set")?;
        Ok(PathBuf::from(format!("{home}/.lab/.env")))
    }
}

/// Interactive `dialoguer` prompt for destructive actions. Returns `true` if
/// the user confirmed. Lives here as a stub — real impl uses `dialoguer` and
/// respects `LAB_CLI_CONFIRM` and `is_terminal::is_terminal(stdin)`.
fn confirm_destructive(action: &str) -> Result<bool> {
    eprintln!("⚠  {action} is destructive. Continue? [y/N]");
    Ok(false)
}
