//! Host-reachability monitor for deployed SSH targets.
//!
//! Probes each target by attempting a TCP connection to its SSH port with a
//! short timeout. Emits one `HostStatusEvent` JSON line to stdout whenever a
//! host transitions between `online` and `offline`, plus an initial snapshot
//! line for every host on startup.
//!
//! Suitable as input for Claude Code's Monitor tool (reads stdout line by line).

use std::collections::HashMap;
use std::io::Write as _;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::net::TcpStream;
use tokio::signal;

use lab_apis::deploy::{HostStatus, HostStatusEvent};

use super::runner::DefaultRunner;
use super::ssh_session::SshHostTarget;

/// Probe a single host by attempting a TCP connect to its SSH port.
async fn probe_host(target: &SshHostTarget, timeout: Duration) -> (HostStatus, String) {
    let host = target.hostname.as_deref().unwrap_or(target.alias.as_str());
    let port = target.port.unwrap_or(22);
    let addr = format!("{host}:{port}");

    let status = match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => HostStatus::Online,
        _ => HostStatus::Offline,
    };
    (status, addr)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(clippy::print_stdout)] // CLI watch streams NDJSON events to stdout by design.
fn emit(event: &HostStatusEvent) {
    let line = serde_json::to_string(event).unwrap_or_default();
    println!("{line}");
    drop(std::io::stdout().flush());
}

/// Watch the given hosts, emitting JSON state-change events to stdout.
///
/// Runs until Ctrl-C is received.
pub async fn watch_hosts(
    runner: &DefaultRunner,
    targets: Vec<String>,
    interval: Duration,
    probe_timeout: Duration,
) {
    let targets: Arc<Vec<SshHostTarget>> = Arc::new(
        targets
            .iter()
            .filter_map(|alias| runner.resolve_target(alias).cloned())
            .collect(),
    );

    // Initial state: assume all hosts offline until first probe.
    let mut states: HashMap<String, HostStatus> = targets
        .iter()
        .map(|t| (t.alias.clone(), HostStatus::Offline))
        .collect();

    // Initial snapshot — probe all hosts once before entering the loop.
    for target in targets.iter() {
        let (status, addr) = probe_host(target, probe_timeout).await;
        states.insert(target.alias.clone(), status);
        emit(&HostStatusEvent {
            ts: now_secs(),
            host: target.alias.clone(),
            status,
            addr,
        });
    }

    let mut ticker = tokio::time::interval(interval);
    ticker.tick().await; // consume the immediate first tick

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                for target in targets.iter() {
                    let (new_status, addr) = probe_host(target, probe_timeout).await;
                    let prev = states.get(&target.alias).copied().unwrap_or(HostStatus::Offline);
                    if new_status != prev {
                        states.insert(target.alias.clone(), new_status);
                        emit(&HostStatusEvent {
                            ts: now_secs(),
                            host: target.alias.clone(),
                            status: new_status,
                            addr,
                        });
                    }
                }
            }
            _ = signal::ctrl_c() => {
                break;
            }
        }
    }
}
