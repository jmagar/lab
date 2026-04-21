use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub struct Config {
    pub webhook_secret: String,
    pub github_token: String,
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub debounce_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let webhook_secret = std::env::var("GH_WEBHOOK_SECRET")
            .context("GH_WEBHOOK_SECRET is required")?;
        if webhook_secret.trim().is_empty() {
            anyhow::bail!("GH_WEBHOOK_SECRET is set but empty; this would accept all traffic");
        }
        let github_token = std::env::var("GH_WEBHOOK_GITHUB_TOKEN")
            .context("GH_WEBHOOK_GITHUB_TOKEN is required")?;
        if github_token.trim().is_empty() {
            anyhow::bail!("GH_WEBHOOK_GITHUB_TOKEN is set but empty");
        }
        let bind = std::env::var("GH_WEBHOOK_BIND")
            .unwrap_or_else(|_| "127.0.0.1:7891".into())
            .parse()
            .context("GH_WEBHOOK_BIND must be host:port")?;
        let data_dir = std::env::var("GH_WEBHOOK_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join(".gh-webhook")
            });
        let debounce_secs = std::env::var("GH_WEBHOOK_DEBOUNCE_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        Ok(Self {
            webhook_secret,
            github_token,
            bind,
            data_dir,
            debounce_secs,
        })
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("webhook_secret", &"[redacted]")
            .field("github_token", &"[redacted]")
            .field("bind", &self.bind)
            .field("data_dir", &self.data_dir)
            .field("debounce_secs", &self.debounce_secs)
            .finish()
    }
}
