//! Local Beads CLI-backed client.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

use super::error::BeadsError;
use super::types::{BdJson, BeadsHealth, ContractStatus};

const DEFAULT_BD_BIN: &str = "bd";

#[derive(Clone, Debug)]
pub struct BeadsClient {
    bd_bin: OsString,
    repo_root: Option<PathBuf>,
}

impl Default for BeadsClient {
    fn default() -> Self {
        Self::new(DEFAULT_BD_BIN, None)
    }
}

impl BeadsClient {
    #[must_use]
    pub fn new(bd_bin: impl Into<OsString>, repo_root: Option<PathBuf>) -> Self {
        Self {
            bd_bin: bd_bin.into(),
            repo_root,
        }
    }

    #[must_use]
    pub fn contract_status(&self) -> ContractStatus {
        ContractStatus {
            status: "cli_contract_implemented",
            reason: "Beads is exposed through the stable local bd CLI JSON contract. Lab v1 is read-only and does not write issues, comments, deps, or Dolt state.",
            safe_v1_actions: &[
                "contract.status",
                "health.status",
                "version.get",
                "context.get",
                "status.summary",
                "issue.list",
                "issue.ready",
                "issue.show",
                "graph.show",
            ],
            deferred: &[
                "issue.create",
                "issue.update",
                "issue.close",
                "comments.add",
                "dep.add",
                "dolt.push",
                "sql.query",
            ],
        }
    }

    pub async fn health_status(&self) -> Result<BeadsHealth, BeadsError> {
        let version = match self.version().await {
            Ok(value) => value.value,
            Err(err) => {
                return Ok(BeadsHealth {
                    bd_available: false,
                    status: "unavailable",
                    version: None,
                    context: None,
                    summary: None,
                    message: Some(err.to_string()),
                });
            }
        };
        let context = self.context().await.ok().map(|json| json.value);
        let summary = self.status_summary().await.ok().map(|json| json.value);
        Ok(BeadsHealth {
            bd_available: true,
            status: "available",
            version: extract_version(&version),
            context,
            summary,
            message: None,
        })
    }

    pub async fn version(&self) -> Result<BdJson, BeadsError> {
        self.run_json("version.get", &["version"]).await
    }

    pub async fn context(&self) -> Result<BdJson, BeadsError> {
        self.run_json("context.get", &["context"]).await
    }

    pub async fn status_summary(&self) -> Result<BdJson, BeadsError> {
        self.run_json("status.summary", &["status"]).await
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        limit: Option<u32>,
    ) -> Result<BdJson, BeadsError> {
        let mut args = vec!["list".to_owned()];
        if let Some(status) = status {
            args.push("--status".to_owned());
            args.push(status.to_owned());
        }
        if let Some(limit) = limit {
            args.push("--limit".to_owned());
            args.push(limit.min(500).to_string());
        }
        self.run_json_owned("issue.list", args).await
    }

    pub async fn ready(&self, limit: Option<u32>) -> Result<BdJson, BeadsError> {
        let mut args = vec!["ready".to_owned()];
        if let Some(limit) = limit {
            args.push("--limit".to_owned());
            args.push(limit.min(500).to_string());
        }
        self.run_json_owned("issue.ready", args).await
    }

    pub async fn show(&self, id: &str) -> Result<BdJson, BeadsError> {
        self.run_json_owned("issue.show", vec!["show".to_owned(), id.to_owned()])
            .await
    }

    pub async fn graph(&self, id: &str) -> Result<BdJson, BeadsError> {
        self.run_json_owned("graph.show", vec!["graph".to_owned(), id.to_owned()])
            .await
    }

    async fn run_json(&self, command: &str, args: &[&str]) -> Result<BdJson, BeadsError> {
        self.run_json_owned(command, args.iter().map(|arg| (*arg).to_owned()).collect())
            .await
    }

    async fn run_json_owned(&self, command: &str, args: Vec<String>) -> Result<BdJson, BeadsError> {
        let client = self.clone();
        let command_name = command.to_owned();
        tokio::task::spawn_blocking(move || client.run_json_blocking(&command_name, args))
            .await
            .map_err(|err| BeadsError::Command {
                command: command.to_owned(),
                message: err.to_string(),
            })?
    }

    fn run_json_blocking(&self, command: &str, args: Vec<String>) -> Result<BdJson, BeadsError> {
        let mut cmd = Command::new(&self.bd_bin);
        if let Some(root) = &self.repo_root {
            cmd.current_dir(root);
        }
        cmd.arg("--json").arg("--readonly").args(&args);
        let output = cmd.output().map_err(|err| BeadsError::Command {
            command: command.to_owned(),
            message: err.to_string(),
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let message = if stderr.is_empty() { stdout } else { stderr };
            return Err(BeadsError::Command {
                command: command.to_owned(),
                message,
            });
        }
        let value =
            serde_json::from_slice::<Value>(&output.stdout).map_err(|err| BeadsError::Decode {
                command: command.to_owned(),
                message: err.to_string(),
            })?;
        Ok(BdJson {
            command: command.to_owned(),
            value,
        })
    }
}

fn extract_version(value: &Value) -> Option<String> {
    value
        .get("version")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_is_read_only() {
        let client = BeadsClient::default();
        let contract = client.contract_status();

        assert!(contract.safe_v1_actions.contains(&"issue.ready"));
        assert!(contract.deferred.contains(&"issue.create"));
    }

    #[test]
    fn extract_version_from_json() {
        let value = serde_json::json!({"version": "0.62.0"});
        assert_eq!(extract_version(&value), Some("0.62.0".to_owned()));
    }
}
