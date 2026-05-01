//! Glances REST API v4 read methods.

use crate::core::{Auth, HttpClient};

use super::error::GlancesError;
use super::types::GlancesResponse;

const MAX_PROCESSES: u32 = 50;

#[derive(Clone)]
pub struct GlancesClient {
    http: HttpClient,
}

impl GlancesClient {
    /// Build a scaffolded client.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, GlancesError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    pub async fn health(&self) -> Result<(), GlancesError> {
        self.http.get_void("/api/4/status").await?;
        Ok(())
    }

    pub async fn plugin(&self, plugin: &str) -> Result<GlancesResponse, GlancesError> {
        if !is_safe_plugin(plugin) {
            return Err(GlancesError::InvalidParam("unsupported plugin".into()));
        }
        Ok(GlancesResponse {
            value: self.http.get_json(&format!("/api/4/{plugin}")).await?,
        })
    }

    pub async fn process_top(&self, limit: u32) -> Result<GlancesResponse, GlancesError> {
        if limit == 0 || limit > MAX_PROCESSES {
            return Err(GlancesError::InvalidParam(format!(
                "limit must be between 1 and {MAX_PROCESSES}"
            )));
        }
        Ok(GlancesResponse {
            value: redact_processes(
                self.http
                    .get_json(&format!("/api/4/processlist/top/{limit}"))
                    .await?,
            ),
        })
    }
}

fn is_safe_plugin(plugin: &str) -> bool {
    matches!(
        plugin,
        "cpu"
            | "mem"
            | "memswap"
            | "load"
            | "network"
            | "diskio"
            | "fs"
            | "containers"
            | "uptime"
            | "quicklook"
            | "system"
    )
}

fn redact_processes(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(items) = value.as_array_mut() {
        for item in items {
            if let Some(map) = item.as_object_mut() {
                map.remove("cmdline");
                map.remove("username");
            }
        }
    }
    value
}
