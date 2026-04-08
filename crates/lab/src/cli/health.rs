//! `lab health` — quick reachability ping for every configured service.

use std::process::ExitCode;

use anyhow::Result;
use serde::Serialize;

use crate::output::{OutputFormat, print};

/// One row of the health report.
#[derive(Debug, Clone, Serialize)]
pub struct HealthRow {
    /// Service identifier.
    pub service: String,
    /// Whether the base URL responded successfully.
    pub reachable: bool,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u64,
}

/// Run the health subcommand. Stub — returns an empty report.
pub async fn run(format: OutputFormat) -> Result<ExitCode> {
    let rows: Vec<HealthRow> = Vec::new();
    print(&rows, format)?;
    Ok(ExitCode::SUCCESS)
}
