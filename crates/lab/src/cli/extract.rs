//! `lab extract` — CLI surface for the extract module.
#![allow(clippy::print_stderr)]
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
use owo_colors::{OwoColorize, XtermColors};

use crate::config::{backup_env, env_is_up_to_date, write_env};
use crate::output::{OutputFormat, print};
use lab_apis::extract::{ExtractClient, ExtractReport, Uri};

/// `lab extract <uri> [--apply | --diff] [-y] [--json]`
#[derive(Debug, Args)]
#[allow(clippy::struct_excessive_bools)]
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

    /// Overwrite conflicting keys instead of skipping them.
    #[arg(long)]
    pub force: bool,

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
        let report = client.scan(uri).await.with_context(|| "scan failed")?;

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
        let target = self.resolve_env_path()?;

        // Rule 8: idempotence check — skip backup and write if nothing would change.
        if env_is_up_to_date(&target, &report.creds) {
            eprintln!("{}", "Already up to date — nothing to write.".dimmed());
            return Ok(());
        }

        self.print_report(report)?;
        eprintln!(
            "\n{} {} {} fields to {}",
            "→".color(XtermColors::LightAzureRadiance),
            "would write".color(XtermColors::LightAzureRadiance).bold(),
            report.creds.len().color(XtermColors::FlushOrange),
            target.display()
        );

        if !self.yes && !self.dry_run && !confirm_destructive("extract.apply")? {
            anyhow::bail!("aborted by user");
        }
        if self.dry_run {
            eprintln!("{}", "(dry-run — no changes written)".dimmed());
            return Ok(());
        }

        // Rule 1: backup before any write.
        let backup =
            backup_env(&target).with_context(|| format!("backup {}", target.display()))?;
        if backup.exists() {
            eprintln!(
                "  {} {}",
                "backup →".dimmed(),
                backup.display().to_string().dimmed()
            );
        }

        // Rules 2–7: atomic merge write.
        let warnings = write_env(&target, &report.creds, self.force)
            .with_context(|| format!("write {}", target.display()))?;

        for w in &warnings {
            eprintln!("  {} {}", "⚠".color(XtermColors::FlushOrange), w);
        }
        eprintln!(
            "  {} {}",
            "✓".color(XtermColors::BrightGreen),
            target.display()
        );
        Ok(())
    }

    fn diff_report(&self, report: &ExtractReport) -> Result<()> {
        let target = self.resolve_env_path()?;

        let existing_raw = if target.exists() {
            std::fs::read_to_string(&target)
                .with_context(|| format!("read {}", target.display()))?
        } else {
            String::new()
        };
        let existing: std::collections::HashMap<String, String> = existing_raw
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .filter_map(|l| {
                l.split_once('=')
                    .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
            })
            .collect();

        let mut any = false;
        for cred in &report.creds {
            let svc_upper = cred.service.to_uppercase();
            if let Some(url) = &cred.url {
                let key = format!("{svc_upper}_URL");
                print_diff_line(&key, url, &existing);
                any = true;
            }
            if let Some(secret) = &cred.secret {
                print_diff_line(&cred.env_field, secret, &existing);
                any = true;
            }
        }

        if !any {
            eprintln!("{}", "No credentials found — nothing to diff.".dimmed());
        }
        Ok(())
    }

    fn print_report(&self, report: &ExtractReport) -> Result<()> {
        let format = OutputFormat::from_json_flag(self.json);
        print(report, format)
    }

    fn resolve_env_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.env_path {
            return Ok(p.clone());
        }
        let home = std::env::var("HOME").context("$HOME not set")?;
        Ok(PathBuf::from(format!("{home}/.lab/.env")))
    }
}

fn print_diff_line(
    key: &str,
    value: &str,
    existing: &std::collections::HashMap<String, String>,
) {
    match existing.get(key) {
        None => eprintln!(
            "  {} {}={}",
            "+".color(XtermColors::BrightGreen).bold(),
            key.color(XtermColors::BrightGreen),
            value
        ),
        Some(ev) if ev == value => {
            // Same value already present — silent in diff output.
        }
        Some(ev) => eprintln!(
            "  {} {} (was {ev:?})",
            "~".color(XtermColors::FlushOrange).bold(),
            format!("{key}={value}").color(XtermColors::FlushOrange)
        ),
    }
}

/// Interactive confirmation prompt for destructive actions.
/// Returns `true` if the user confirmed, `false` otherwise.
fn confirm_destructive(action: &str) -> Result<bool> {
    use is_terminal::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }
    eprintln!(
        "{} {} is destructive. Continue? [y/N]",
        "⚠".color(XtermColors::FlushOrange),
        action.color(XtermColors::LightAzureRadiance).bold(),
    );
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .context("read confirmation")?;
    Ok(buf.trim().eq_ignore_ascii_case("y"))
}
