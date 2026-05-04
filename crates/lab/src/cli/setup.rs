//! `labby setup` — first-run wizard entry point.
//!
//! Thin CLI shim over the `setup` dispatch service. Detects first-run via
//! `setup.state`, then prints either:
//!
//! - first-run: instructions to start `labby serve` and visit `/setup`, or
//! - re-run: instructions to visit `/settings`.
//!
//! Honors `LAB_SKIP_SETUP=1` and `--no-setup` for CI / power users.
//!
//! Browser auto-launch is intentionally deferred to a follow-up so this PR
//! avoids adding the `webbrowser` dependency. The bead's locked decision
//! includes browser launch + headless detection; that wiring can land
//! incrementally without breaking the CLI surface contract.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;
use serde_json::{Value, json};

#[derive(Debug, Args)]
pub struct SetupArgs {
    /// Skip the wizard and exit cleanly. Equivalent to LAB_SKIP_SETUP=1.
    #[arg(long)]
    pub no_setup: bool,

    /// Do not attempt to open the browser (no-op for now; reserved for
    /// the follow-up that adds `webbrowser` integration).
    #[arg(long)]
    pub no_browser: bool,

    /// Smoke-test mode: print the state machine snapshot as JSON and exit.
    /// Used by `just smoke-setup` for CI verification.
    #[arg(long)]
    pub smoke: bool,
}

/// Default URL for the embedded web UI (per Q1: 127.0.0.1:8765).
const DEFAULT_LAB_URL: &str = "http://127.0.0.1:8765";

pub async fn run(args: SetupArgs) -> Result<ExitCode> {
    if std::env::var("LAB_SKIP_SETUP").as_deref() == Ok("1") || args.no_setup {
        eprintln!(
            "setup skipped (LAB_SKIP_SETUP=1 or --no-setup); run `labby setup` manually when ready"
        );
        return Ok(ExitCode::SUCCESS);
    }

    let snapshot = crate::dispatch::setup::dispatch("state", json!({}))
        .await
        .map_err(|e| anyhow::anyhow!("setup.state failed: {e:?}"))?;

    if args.smoke {
        println!(
            "{}",
            serde_json::to_string_pretty(&snapshot).unwrap_or_default()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let first_run = snapshot
        .get("first_run")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let route = if first_run { "/setup" } else { "/settings" };

    let url = format!("{DEFAULT_LAB_URL}{route}");
    eprintln!();
    if first_run {
        eprintln!("Welcome to lab. First-run detected.");
    } else {
        eprintln!("lab is already configured. Opening Settings.");
    }
    eprintln!();
    eprintln!("→ Run `labby serve` and visit: {url}");
    eprintln!();
    eprintln!("Tip: set LAB_SKIP_SETUP=1 to suppress this message in CI.");
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_setup_flag_exits_cleanly() {
        let code = run(SetupArgs {
            no_setup: true,
            no_browser: true,
            smoke: false,
        })
        .await
        .unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
