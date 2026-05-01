//! Service probe logic for `service.probe` and `audit.full`.
//!
//! URL resolution comes exclusively from pre-built `ServiceClients` (which were
//! constructed from env at startup). Caller-supplied URLs in params are rejected
//! by `params.rs`. This is the SSRF defense boundary.

use std::sync::Arc;

use lab_apis::core::{ServiceClient, ServiceStatus};
use tokio::sync::Semaphore;

use crate::dispatch::clients::ServiceClients;
use crate::dispatch::error::ToolError;

use super::types::{Finding, Severity};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Probe a single named service and return a `Finding`.
///
/// Returns `Err` when the service name is not in the known service list.
pub async fn probe_service(
    clients: &ServiceClients,
    service: &str,
    _instance: Option<&str>,
) -> Result<Finding, ToolError> {
    let all_names = all_service_names();
    if !all_names.contains(&service) {
        return Err(ToolError::InvalidParam {
            message: format!("unknown service `{service}`"),
            param: "service".to_string(),
        });
    }
    let status = health_by_name_owned(clients, service).await;
    Ok(status_to_finding(service, &status))
}

/// Run only service probes, without system or auth checks.
///
/// Used by `lab doctor services`. Results are sent to `tx` as they complete,
/// in parallel bounded by `Semaphore(5)`.
pub async fn stream_service_probes(
    clients: Arc<ServiceClients>,
    tx: tokio::sync::mpsc::Sender<Finding>,
) {
    let sem = Arc::new(Semaphore::new(5));
    let mut handles = Vec::new();

    for service_name in configured_service_names(&clients) {
        let sem = sem.clone();
        let clients = clients.clone();
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let status = health_by_name_owned(&clients, &service_name);
            let finding = status_to_finding(&service_name, &status.await);
            tx.send(finding).await.ok();
        }));
    }

    for handle in handles {
        handle.await.ok();
    }
}

/// Run `audit.full`: system checks followed by all configured service probes.
///
/// Results are sent to `tx` as they complete. System checks are emitted
/// synchronously first; service probes run in parallel bounded by `Semaphore(5)`.
pub async fn stream_audit_full(
    clients: Arc<ServiceClients>,
    tx: tokio::sync::mpsc::Sender<Finding>,
) {
    // Emit system and auth checks immediately (no network I/O).
    for finding in super::system::run_system_checks() {
        if tx.send(finding).await.is_err() {
            return;
        }
    }
    for finding in super::system::run_auth_checks() {
        if tx.send(finding).await.is_err() {
            return;
        }
    }

    let sem = Arc::new(Semaphore::new(5));
    let mut handles = Vec::new();

    for service_name in configured_service_names(&clients) {
        let sem = sem.clone();
        let clients = clients.clone();
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let status = health_by_name_owned(&clients, &service_name);
            let finding = status_to_finding(&service_name, &status.await);
            tx.send(finding).await.ok();
        }));
    }

    for handle in handles {
        handle.await.ok();
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// All known service names (compiled in), regardless of configuration.
fn all_service_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    #[cfg(feature = "radarr")]
    {
        names.push("radarr");
    }
    #[cfg(feature = "sonarr")]
    {
        names.push("sonarr");
    }
    #[cfg(feature = "prowlarr")]
    {
        names.push("prowlarr");
    }
    #[cfg(feature = "plex")]
    {
        names.push("plex");
    }
    #[cfg(feature = "tautulli")]
    {
        names.push("tautulli");
    }
    #[cfg(feature = "sabnzbd")]
    {
        names.push("sabnzbd");
    }
    #[cfg(feature = "qbittorrent")]
    {
        names.push("qbittorrent");
    }
    #[cfg(feature = "tailscale")]
    {
        names.push("tailscale");
    }
    #[cfg(feature = "linkding")]
    {
        names.push("linkding");
    }
    #[cfg(feature = "memos")]
    {
        names.push("memos");
    }
    #[cfg(feature = "bytestash")]
    {
        names.push("bytestash");
    }
    #[cfg(feature = "paperless")]
    {
        names.push("paperless");
    }
    #[cfg(feature = "arcane")]
    {
        names.push("arcane");
    }
    #[cfg(feature = "unraid")]
    {
        names.push("unraid");
    }
    #[cfg(feature = "overseerr")]
    {
        names.push("overseerr");
    }
    #[cfg(feature = "gotify")]
    {
        names.push("gotify");
    }
    #[cfg(feature = "openai")]
    {
        names.push("openai");
    }
    #[cfg(feature = "notebooklm")]
    {
        names.push("notebooklm");
    }
    #[cfg(feature = "qdrant")]
    {
        names.push("qdrant");
    }
    #[cfg(feature = "tei")]
    {
        names.push("tei");
    }
    #[cfg(feature = "apprise")]
    {
        names.push("apprise");
    }
    names
}

/// Names of services that have a configured (non-None) client.
fn configured_service_names(clients: &ServiceClients) -> Vec<String> {
    let mut names = Vec::new();
    #[cfg(feature = "radarr")]
    {
        if clients.radarr.is_some() {
            names.push("radarr".into());
        }
    }
    #[cfg(feature = "sonarr")]
    {
        if clients.sonarr.is_some() {
            names.push("sonarr".into());
        }
    }
    #[cfg(feature = "prowlarr")]
    {
        if clients.prowlarr.is_some() {
            names.push("prowlarr".into());
        }
    }
    #[cfg(feature = "plex")]
    {
        if clients.plex.is_some() {
            names.push("plex".into());
        }
    }
    #[cfg(feature = "tautulli")]
    {
        if clients.tautulli.is_some() {
            names.push("tautulli".into());
        }
    }
    #[cfg(feature = "sabnzbd")]
    {
        if clients.sabnzbd.is_some() {
            names.push("sabnzbd".into());
        }
    }
    #[cfg(feature = "qbittorrent")]
    {
        if clients.qbittorrent.is_some() {
            names.push("qbittorrent".into());
        }
    }
    #[cfg(feature = "tailscale")]
    {
        if clients.tailscale.is_some() {
            names.push("tailscale".into());
        }
    }
    #[cfg(feature = "linkding")]
    {
        if clients.linkding.is_some() {
            names.push("linkding".into());
        }
    }
    #[cfg(feature = "memos")]
    {
        if clients.memos.is_some() {
            names.push("memos".into());
        }
    }
    #[cfg(feature = "bytestash")]
    {
        if clients.bytestash.is_some() {
            names.push("bytestash".into());
        }
    }
    #[cfg(feature = "paperless")]
    {
        if clients.paperless.is_some() {
            names.push("paperless".into());
        }
    }
    #[cfg(feature = "arcane")]
    {
        if clients.arcane.is_some() {
            names.push("arcane".into());
        }
    }
    #[cfg(feature = "unraid")]
    {
        if clients.unraid.is_some() {
            names.push("unraid".into());
        }
    }
    #[cfg(feature = "overseerr")]
    {
        if clients.overseerr.is_some() {
            names.push("overseerr".into());
        }
    }
    #[cfg(feature = "gotify")]
    {
        if clients.gotify.is_some() {
            names.push("gotify".into());
        }
    }
    #[cfg(feature = "openai")]
    {
        if clients.openai.is_some() {
            names.push("openai".into());
        }
    }
    #[cfg(feature = "notebooklm")]
    {
        if clients.notebooklm.is_some() {
            names.push("notebooklm".into());
        }
    }
    #[cfg(feature = "qdrant")]
    {
        if clients.qdrant.is_some() {
            names.push("qdrant".into());
        }
    }
    #[cfg(feature = "tei")]
    {
        if clients.tei.is_some() {
            names.push("tei".into());
        }
    }
    #[cfg(feature = "apprise")]
    {
        if clients.apprise.is_some() {
            names.push("apprise".into());
        }
    }
    names
}

/// Async version suitable for use inside `tokio::spawn`.
async fn health_by_name_owned(clients: &ServiceClients, service: &str) -> ServiceStatus {
    match service {
        #[cfg(feature = "radarr")]
        "radarr" => probe_arc(clients.radarr.as_ref()).await,
        #[cfg(feature = "sonarr")]
        "sonarr" => probe_arc(clients.sonarr.as_ref()).await,
        #[cfg(feature = "prowlarr")]
        "prowlarr" => probe_arc(clients.prowlarr.as_ref()).await,
        #[cfg(feature = "plex")]
        "plex" => probe_arc(clients.plex.as_ref()).await,
        #[cfg(feature = "tautulli")]
        "tautulli" => probe_arc(clients.tautulli.as_ref()).await,
        #[cfg(feature = "sabnzbd")]
        "sabnzbd" => probe_arc(clients.sabnzbd.as_ref()).await,
        #[cfg(feature = "qbittorrent")]
        "qbittorrent" => probe_arc(clients.qbittorrent.as_ref()).await,
        #[cfg(feature = "tailscale")]
        "tailscale" => probe_arc(clients.tailscale.as_ref()).await,
        #[cfg(feature = "linkding")]
        "linkding" => probe_arc(clients.linkding.as_ref()).await,
        #[cfg(feature = "memos")]
        "memos" => probe_arc(clients.memos.as_ref()).await,
        #[cfg(feature = "bytestash")]
        "bytestash" => probe_arc(clients.bytestash.as_ref()).await,
        #[cfg(feature = "paperless")]
        "paperless" => probe_arc(clients.paperless.as_ref()).await,
        #[cfg(feature = "arcane")]
        "arcane" => probe_arc(clients.arcane.as_ref()).await,
        #[cfg(feature = "unraid")]
        "unraid" => probe_arc(clients.unraid.as_ref()).await,
        #[cfg(feature = "overseerr")]
        "overseerr" => probe_arc(clients.overseerr.as_ref()).await,
        #[cfg(feature = "gotify")]
        "gotify" => match &clients.gotify {
            Some(gc) => gc
                .health()
                .health()
                .await
                .unwrap_or_else(|e| ServiceStatus::unreachable(e.to_string())),
            None => ServiceStatus::unreachable("not configured"),
        },
        #[cfg(feature = "openai")]
        "openai" => probe_arc(clients.openai.as_ref()).await,
        #[cfg(feature = "notebooklm")]
        "notebooklm" => probe_arc(clients.notebooklm.as_ref()).await,
        #[cfg(feature = "qdrant")]
        "qdrant" => probe_arc(clients.qdrant.as_ref()).await,
        #[cfg(feature = "tei")]
        "tei" => probe_arc(clients.tei.as_ref()).await,
        #[cfg(feature = "apprise")]
        "apprise" => probe_arc(clients.apprise.as_ref()).await,
        _ => ServiceStatus::unreachable(format!("unknown service `{service}`")),
    }
}

async fn probe_arc<C: ServiceClient>(client: Option<&Arc<C>>) -> ServiceStatus {
    match client {
        Some(c) => c
            .health()
            .await
            .unwrap_or_else(|e| ServiceStatus::unreachable(e.to_string())),
        None => ServiceStatus::unreachable("not configured"),
    }
}

fn status_to_finding(service: &str, status: &ServiceStatus) -> Finding {
    let severity = if !status.reachable {
        Severity::Fail
    } else if !status.auth_ok {
        Severity::Warn
    } else {
        Severity::Ok
    };
    let message = match (&status.message, status.reachable, status.auth_ok) {
        (Some(msg), _, _) => {
            // Truncate to prevent HTML error pages from flooding output.
            let msg = msg.trim();
            let truncated = if msg.chars().count() > 120 {
                let prefix: String = msg.chars().take(120).collect();
                format!("{prefix}…")
            } else {
                msg.to_string()
            };
            format!("{truncated} ({}ms)", status.latency_ms)
        }
        (None, true, true) => format!("healthy ({}ms)", status.latency_ms),
        (None, true, false) => format!("reachable but auth failed ({}ms)", status.latency_ms),
        (None, false, _) => "unreachable".to_string(),
    };
    Finding {
        service: service.to_string(),
        check: "health".to_string(),
        severity,
        message,
    }
}
