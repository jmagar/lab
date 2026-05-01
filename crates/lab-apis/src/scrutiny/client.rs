//! Scrutiny read-only API methods.

use crate::core::{Auth, HttpClient};

use super::error::ScrutinyError;
use super::types::ScrutinyResponse;

#[derive(Clone)]
pub struct ScrutinyClient {
    http: HttpClient,
}

impl ScrutinyClient {
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ScrutinyError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    pub async fn health(&self) -> Result<(), ScrutinyError> {
        self.http.get_void("/api/health").await?;
        Ok(())
    }

    pub async fn dashboard_summary(&self) -> Result<ScrutinyResponse, ScrutinyError> {
        let value = self.http.get_json("/api/summary").await?;
        Ok(ScrutinyResponse {
            value: redact_disk_value(value),
        })
    }

    pub async fn device_list(&self) -> Result<ScrutinyResponse, ScrutinyError> {
        let value = self.http.get_json("/api/devices").await?;
        Ok(ScrutinyResponse {
            value: redact_disk_value(value),
        })
    }
}

fn redact_disk_value(mut value: serde_json::Value) -> serde_json::Value {
    redact_recursive(&mut value);
    value
}

fn redact_recursive(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in [
                "serial",
                "serial_number",
                "wwn",
                "device",
                "device_name",
                "device_path",
                "host",
                "hostname",
                "smart_results",
                "smartctl",
            ] {
                map.remove(key);
            }
            for child in map.values_mut() {
                redact_recursive(child);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                redact_recursive(item);
            }
        }
        _ => {}
    }
}
