//! `lab health` — quick reachability ping for every configured service.

use std::process::ExitCode;

use anyhow::Result;
use serde::Serialize;

use crate::output::{OutputFormat, print};

/// One row of the health report.
#[derive(Debug, Clone, Serialize)]
pub struct HealthRow {
    pub service: String,
    pub reachable: bool,
    pub auth_ok: bool,
    pub version: Option<String>,
    pub latency_ms: u64,
    pub message: Option<String>,
}

impl HealthRow {
    fn not_configured(service: &str) -> Self {
        Self {
            service: service.into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some("not configured".into()),
        }
    }
}

/// Run the health subcommand.
pub async fn run(format: OutputFormat) -> Result<ExitCode> {
    let mut rows: Vec<HealthRow> = Vec::new();

    // extract has no network endpoint — always available.
    rows.push(HealthRow {
        service: "extract".into(),
        reachable: true,
        auth_ok: true,
        version: None,
        latency_ms: 0,
        message: Some("local scan service (always available)".into()),
    });

    #[cfg(feature = "radarr")]
    rows.push(radarr_row().await);

    // Stub services — report not-configured until client is implemented.
    // Remove a service from this list once its client_from_env() is wired.
    #[cfg(feature = "sonarr")]
    rows.push(HealthRow::not_configured("sonarr"));
    #[cfg(feature = "prowlarr")]
    rows.push(HealthRow::not_configured("prowlarr"));
    #[cfg(feature = "plex")]
    rows.push(HealthRow::not_configured("plex"));
    #[cfg(feature = "tautulli")]
    rows.push(HealthRow::not_configured("tautulli"));
    #[cfg(feature = "sabnzbd")]
    rows.push(sabnzbd_row().await);
    #[cfg(feature = "qbittorrent")]
    rows.push(HealthRow::not_configured("qbittorrent"));
    #[cfg(feature = "tailscale")]
    rows.push(HealthRow::not_configured("tailscale"));
    #[cfg(feature = "linkding")]
    rows.push(HealthRow::not_configured("linkding"));
    #[cfg(feature = "memos")]
    rows.push(HealthRow::not_configured("memos"));
    #[cfg(feature = "bytestash")]
    rows.push(bytestash_row().await);
    #[cfg(feature = "paperless")]
    rows.push(HealthRow::not_configured("paperless"));
    #[cfg(feature = "arcane")]
    rows.push(HealthRow::not_configured("arcane"));
    #[cfg(feature = "unraid")]
    rows.push(unraid_row().await);
    #[cfg(feature = "unifi")]
    rows.push(unifi_row().await);
    #[cfg(feature = "overseerr")]
    rows.push(HealthRow::not_configured("overseerr"));
    #[cfg(feature = "gotify")]
    rows.push(gotify_row().await);
    #[cfg(feature = "openai")]
    rows.push(HealthRow::not_configured("openai"));
    #[cfg(feature = "qdrant")]
    rows.push(HealthRow::not_configured("qdrant"));
    #[cfg(feature = "tei")]
    rows.push(HealthRow::not_configured("tei"));
    #[cfg(feature = "apprise")]
    rows.push(HealthRow::not_configured("apprise"));

    let any_unhealthy = rows.iter().any(|r| !r.reachable || !r.auth_ok);
    print(&rows, format)?;
    if any_unhealthy {
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

/// Build a [`HealthRow`] for any service that implements [`lab_apis::core::ServiceClient`].
///
/// Centralises the not-configured / ok / error branching that was repeated
/// verbatim for every service.
async fn service_health_row<C>(
    service: &str,
    client: Option<C>,
    not_configured_msg: &str,
) -> HealthRow
where
    C: lab_apis::core::ServiceClient,
{
    let Some(client) = client else {
        return HealthRow {
            service: service.into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some(not_configured_msg.into()),
        };
    };
    let start = std::time::Instant::now();
    let row = match client.health().await {
        Ok(s) => HealthRow {
            service: service.into(),
            reachable: s.reachable,
            auth_ok: s.auth_ok,
            version: s.version,
            latency_ms: s.latency_ms,
            message: s.message,
        },
        Err(e) => HealthRow {
            service: service.into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some(e.to_string()),
        },
    };
    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    if row.reachable && row.auth_ok {
        tracing::info!(
            surface = "cli",
            service,
            operation = "health",
            elapsed_ms,
            "health ok"
        );
    } else {
        tracing::warn!(
            surface = "cli",
            service,
            operation = "health",
            elapsed_ms,
            "health issue"
        );
    }
    row
}

#[cfg(feature = "radarr")]
async fn radarr_row() -> HealthRow {
    service_health_row(
        "radarr",
        crate::dispatch::radarr::client_from_env(),
        "RADARR_URL / RADARR_API_KEY not set",
    )
    .await
}

#[cfg(feature = "unifi")]
async fn unifi_row() -> HealthRow {
    service_health_row(
        "unifi",
        crate::dispatch::unifi::client_from_env(),
        "UNIFI_URL / UNIFI_API_KEY not set",
    )
    .await
}

#[cfg(feature = "gotify")]
async fn gotify_row() -> HealthRow {
    service_health_row(
        "gotify",
        crate::dispatch::gotify::client_from_env(),
        "GOTIFY_URL / GOTIFY_CLIENT_TOKEN (or GOTIFY_TOKEN) not set",
    )
    .await
}

#[cfg(feature = "unraid")]
async fn unraid_row() -> HealthRow {
    service_health_row(
        "unraid",
        crate::dispatch::unraid::client_from_env(),
        "UNRAID_URL / UNRAID_API_KEY not set",
    )
    .await
}

#[cfg(feature = "sabnzbd")]
async fn sabnzbd_row() -> HealthRow {
    service_health_row(
        "sabnzbd",
        crate::dispatch::sabnzbd::client_from_env(),
        "SABNZBD_URL / SABNZBD_API_KEY not set",
    )
    .await
}

#[cfg(feature = "bytestash")]
async fn bytestash_row() -> HealthRow {
    service_health_row(
        "bytestash",
        crate::dispatch::bytestash::client_from_env(),
        "BYTESTASH_URL / BYTESTASH_TOKEN not set",
    )
    .await
}
