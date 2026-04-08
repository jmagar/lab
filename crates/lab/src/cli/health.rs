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
    rows.push(HealthRow::not_configured("sabnzbd"));
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
    rows.push(HealthRow::not_configured("unraid"));
    #[cfg(feature = "unifi")]
    rows.push(unifi_row().await);
    #[cfg(feature = "overseerr")]
    rows.push(HealthRow::not_configured("overseerr"));
    #[cfg(feature = "gotify")]
    rows.push(HealthRow::not_configured("gotify"));
    #[cfg(feature = "openai")]
    rows.push(HealthRow::not_configured("openai"));
    #[cfg(feature = "qdrant")]
    rows.push(HealthRow::not_configured("qdrant"));
    #[cfg(feature = "tei")]
    rows.push(HealthRow::not_configured("tei"));
    #[cfg(feature = "apprise")]
    rows.push(HealthRow::not_configured("apprise"));

    print(&rows, format)?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(feature = "radarr")]
async fn radarr_row() -> HealthRow {
    use lab_apis::core::ServiceClient;

    let Some(client) = crate::mcp::services::radarr::client_from_env() else {
        return HealthRow {
            service: "radarr".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some("RADARR_URL / RADARR_API_KEY not set".into()),
        };
    };

    match <_ as ServiceClient>::health(&client).await {
        Ok(s) => HealthRow {
            service: "radarr".into(),
            reachable: s.reachable,
            auth_ok: s.auth_ok,
            version: s.version,
            latency_ms: s.latency_ms,
            message: s.message,
        },
        Err(e) => HealthRow {
            service: "radarr".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some(e.to_string()),
        },
    }
}

#[cfg(feature = "unifi")]
async fn unifi_row() -> HealthRow {
    use lab_apis::core::ServiceClient;

    let Some(client) = crate::mcp::services::unifi::client_from_env() else {
        return HealthRow {
            service: "unifi".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some("UNIFI_URL / UNIFI_API_KEY not set".into()),
        };
    };

    match <_ as ServiceClient>::health(&client).await {
        Ok(s) => HealthRow {
            service: "unifi".into(),
            reachable: s.reachable,
            auth_ok: s.auth_ok,
            version: s.version,
            latency_ms: s.latency_ms,
            message: s.message,
        },
        Err(e) => HealthRow {
            service: "unifi".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some(e.to_string()),
        },
    }
}

#[cfg(feature = "bytestash")]
async fn bytestash_row() -> HealthRow {
    use lab_apis::core::ServiceClient;

    let Some(client) = crate::mcp::services::bytestash::client_from_env() else {
        return HealthRow {
            service: "bytestash".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some("BYTESTASH_URL / BYTESTASH_TOKEN not set".into()),
        };
    };

    match <_ as ServiceClient>::health(&client).await {
        Ok(s) => HealthRow {
            service: "bytestash".into(),
            reachable: s.reachable,
            auth_ok: s.auth_ok,
            version: s.version,
            latency_ms: s.latency_ms,
            message: s.message,
        },
        Err(e) => HealthRow {
            service: "bytestash".into(),
            reachable: false,
            auth_ok: false,
            version: None,
            latency_ms: 0,
            message: Some(e.to_string()),
        },
    }
}
