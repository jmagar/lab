use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedEnvelope {
    pub kind: String,
    pub payload: serde_json::Value,
}

impl QueuedEnvelope {
    #[must_use]
    pub fn status(payload: serde_json::Value) -> Self {
        Self {
            kind: "status".to_string(),
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeviceOutboundQueue {
    path: PathBuf,
    entries: Arc<Mutex<Vec<QueuedEnvelope>>>,
}

impl DeviceOutboundQueue {
    pub async fn open(path: PathBuf) -> Result<Self> {
        let entries = read_entries(&path).await?;
        Ok(Self {
            path,
            entries: Arc::new(Mutex::new(entries)),
        })
    }

    pub async fn push(&self, envelope: QueuedEnvelope) -> Result<()> {
        let mut entries = self.entries.lock().await;
        entries.push(envelope.clone());
        append_entry(&self.path, &envelope).await
    }

    pub async fn drain_batch(&self, limit: usize) -> Result<Vec<QueuedEnvelope>> {
        let entries = self.entries.lock().await;
        Ok(entries.iter().take(limit).cloned().collect())
    }

    pub async fn ack_drained(&self, count: usize) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let drained = count.min(entries.len());
        entries.drain(..drained);
        rewrite_entries(&self.path, &entries).await
    }
}

async fn read_entries(path: &Path) -> Result<Vec<QueuedEnvelope>> {
    if !fs::try_exists(path).await? {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path)
        .await
        .with_context(|| format!("read {}", path.display()))?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).context("parse queue entry"))
        .collect()
}

async fn append_entry(path: &Path, envelope: &QueuedEnvelope) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let mut line =
        serde_json::to_string(envelope).context("serialize queue entry for append")?;
    line.push('\n');

    use tokio::io::AsyncWriteExt as _;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .with_context(|| format!("open {}", path.display()))?;
    file.write_all(line.as_bytes())
        .await
        .with_context(|| format!("append {}", path.display()))
}

async fn rewrite_entries(path: &Path, entries: &[QueuedEnvelope]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let content = if entries.is_empty() {
        String::new()
    } else {
        let mut serialized = entries
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()
            .context("serialize queue entries for rewrite")?
            .join("\n");
        serialized.push('\n');
        serialized
    };

    fs::write(path, content)
        .await
        .with_context(|| format!("rewrite {}", path.display()))
}
