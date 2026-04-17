//! File-reading transports: local fs and ssh.
//!
//! Uses **enum dispatch** instead of `dyn Transport` because the set of
//! transports is closed (two variants), and enum dispatch keeps the public
//! API object-safety-friendly without `async-trait`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use russh::keys::agent::AgentIdentity;
use russh::keys::agent::client::AgentClient;
use russh::{ChannelMsg, client};
use russh_sftp::client::SftpSession;

use super::error::ExtractError;

/// File-reading transport — wraps either local filesystem access or an
/// authenticated SSH session.
#[derive(Debug)]
pub enum Transport {
    /// Read files from the local filesystem with `tokio::fs`.
    Local(LocalFs),
    /// Read files from a remote host over SSH (`russh`).
    Ssh(SshFs),
}

/// Captured output from a remote command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: Option<u32>,
}

impl Transport {
    /// Read a file at `path`, returning its raw bytes.
    ///
    /// # Errors
    /// Returns `ExtractError::Io` for local read failures and
    /// `ExtractError::Ssh` for remote failures.
    pub async fn read(&self, path: &Path) -> Result<Vec<u8>, ExtractError> {
        match self {
            Self::Local(t) => t.read(path).await,
            Self::Ssh(t) => t.read(path).await,
        }
    }

    /// List entries in `dir`, returning subdirectory names only (used by the
    /// scanner to enumerate per-app appdata folders).
    ///
    /// # Errors
    /// Same as [`Transport::read`].
    pub async fn list_subdirs(&self, dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        match self {
            Self::Local(t) => t.list_subdirs(dir).await,
            Self::Ssh(t) => t.list_subdirs(dir).await,
        }
    }
}

/// Local filesystem transport. No state — every call hits `tokio::fs`.
#[derive(Debug, Default)]
pub struct LocalFs;

impl LocalFs {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, ExtractError> {
        tokio::fs::read(path)
            .await
            .map_err(|source| ExtractError::Io {
                path: path.to_path_buf(),
                source,
            })
    }

    async fn list_subdirs(&self, dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        let mut read_dir = tokio::fs::read_dir(dir)
            .await
            .map_err(|source| ExtractError::Io {
                path: dir.to_path_buf(),
                source,
            })?;

        let mut entries = Vec::new();
        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|source| ExtractError::Io {
                path: dir.to_path_buf(),
                source,
            })?
        {
            let ft = entry.file_type().await.map_err(|source| ExtractError::Io {
                path: entry.path(),
                source,
            })?;
            if ft.is_dir() {
                entries.push(entry.path());
            }
        }
        Ok(entries)
    }
}

/// russh `Handler` that trusts all server keys — appropriate for private
/// homelab hosts where TOFU (trust on first use) is acceptable.
struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept every host key — homelab use only.
        Ok(true)
    }
}

/// SSH transport — opens a `russh` session to a host alias resolved via
/// `~/.ssh/config`, runs an SFTP subsystem, and reads files over the channel.
///
/// The inner `russh` `Handle` is kept alive for the lifetime of this struct so
/// the SFTP channel remains open.
pub struct SshFs {
    /// SSH host alias (for error messages).
    host: String,
    /// Live SFTP session.
    sftp: SftpSession,
    /// Keeps the underlying SSH connection open as long as `SshFs` lives.
    _session: client::Handle<ClientHandler>,
}

impl std::fmt::Debug for SshFs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SshFs")
            .field("host", &self.host)
            .finish_non_exhaustive()
    }
}

impl SshFs {
    /// Open a new SSH connection. Authentication uses whatever keys the
    /// running `ssh-agent` (via `SSH_AUTH_SOCK`) has loaded. The host alias
    /// is resolved through `~/.ssh/config` — supports `Hostname`, `Port`,
    /// `User`, and `ProxyCommand` directives.
    ///
    /// # Errors
    /// Returns `ExtractError::Ssh` on config parse, connection, auth, or
    /// SFTP subsystem failures.
    pub async fn connect(host: impl Into<String>) -> Result<Self, ExtractError> {
        let host = host.into();

        // Resolve the alias through ~/.ssh/config (Hostname, Port, User, ProxyCommand).
        let cfg = russh_config::parse_home(&host).map_err(|e| ExtractError::Ssh {
            host: host.clone(),
            message: format!("ssh config: {e}"),
        })?;
        let user = cfg.user();

        // Open TCP stream (or ProxyCommand pipe) to the resolved endpoint.
        let stream = cfg.stream().await.map_err(|e| ExtractError::Ssh {
            host: host.clone(),
            message: format!("connect: {e}"),
        })?;

        // Perform the SSH handshake.
        let ssh_cfg = Arc::new(client::Config::default());
        let mut handle = client::connect_stream(ssh_cfg, stream, ClientHandler)
            .await
            .map_err(|e| ExtractError::Ssh {
                host: host.clone(),
                message: format!("handshake: {e}"),
            })?;

        // Authenticate using the running ssh-agent.
        let mut agent = AgentClient::connect_env()
            .await
            .map_err(|e| ExtractError::Ssh {
                host: host.clone(),
                message: format!("ssh-agent: {e}"),
            })?;

        let identities = agent
            .request_identities()
            .await
            .map_err(|e| ExtractError::Ssh {
                host: host.clone(),
                message: format!("agent identities: {e}"),
            })?;

        let mut authenticated = false;
        for identity in &identities {
            // Only handle plain public-key identities; skip certificates.
            let pub_key = match identity {
                AgentIdentity::PublicKey { key, .. } => key.clone(),
                AgentIdentity::Certificate { .. } => continue,
            };

            let result = handle
                .authenticate_publickey_with(&user, pub_key, None, &mut agent)
                .await
                .map_err(|e| ExtractError::Ssh {
                    host: host.clone(),
                    message: format!("agent auth: {e:?}"),
                })?;

            if result.success() {
                authenticated = true;
                break;
            }
        }

        if !authenticated {
            return Err(ExtractError::Ssh {
                host: host.clone(),
                message: "all agent identities were rejected".to_owned(),
            });
        }

        // Open an SSH session channel and request the SFTP subsystem.
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| ExtractError::Ssh {
                host: host.clone(),
                message: format!("channel open: {e}"),
            })?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| ExtractError::Ssh {
                host: host.clone(),
                message: format!("sftp subsystem: {e}"),
            })?;

        let sftp =
            SftpSession::new(channel.into_stream())
                .await
                .map_err(|e| ExtractError::Ssh {
                    host: host.clone(),
                    message: format!("sftp init: {e}"),
                })?;

        Ok(Self {
            host,
            sftp,
            _session: handle,
        })
    }

    pub(crate) async fn read(&self, path: &Path) -> Result<Vec<u8>, ExtractError> {
        self.sftp
            .read(path.to_string_lossy().into_owned())
            .await
            .map_err(|e| ExtractError::Ssh {
                host: self.host.clone(),
                message: format!("sftp read: {e}"),
            })
    }

    async fn list_subdirs(&self, dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        let read_dir = self
            .sftp
            .read_dir(dir.to_string_lossy().into_owned())
            .await
            .map_err(|e| ExtractError::Ssh {
                host: self.host.clone(),
                message: format!("sftp readdir: {e}"),
            })?;

        Ok(read_dir
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| dir.join(entry.file_name()))
            .collect())
    }

    #[allow(dead_code)]
    pub(crate) async fn run_command(
        &self,
        action: &str,
        command: &str,
    ) -> Result<RemoteCommandOutput, ExtractError> {
        let mut channel =
            self._session
                .channel_open_session()
                .await
                .map_err(|e| ExtractError::Ssh {
                    host: self.host.clone(),
                    message: format!("channel open: {e}"),
                })?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| ExtractError::Ssh {
                host: self.host.clone(),
                message: format!("exec: {e}"),
            })?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut exit_status = None;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => stdout.push_str(&String::from_utf8_lossy(&data)),
                ChannelMsg::ExtendedData { data, .. } => {
                    stderr.push_str(&String::from_utf8_lossy(&data));
                }
                ChannelMsg::ExitStatus {
                    exit_status: status,
                } => exit_status = Some(status),
                _ => {}
            }
        }

        let output = RemoteCommandOutput {
            stdout,
            stderr,
            exit_status,
        };

        if output.exit_status != Some(0) {
            return Err(remote_command_failed(&self.host, action, command, output));
        }

        Ok(output)
    }
}

fn remote_command_failed(
    host: &str,
    action: &str,
    _command: &str,
    output: RemoteCommandOutput,
) -> ExtractError {
    let summary = output
        .stderr
        .trim()
        .to_owned()
        .or_else_if_empty(|| output.stdout.trim().to_owned())
        .unwrap_or_else(|| match output.exit_status {
            Some(_) => "remote command returned a non-zero exit status".to_owned(),
            None => "remote command did not report an exit status".to_owned(),
        });

    ExtractError::RemoteCommand {
        host: host.to_owned(),
        action: action.to_owned(),
        exit_status: output.exit_status,
        message: summary,
    }
}

trait StringFallback {
    fn or_else_if_empty<F>(self, fallback: F) -> Option<String>
    where
        F: FnOnce() -> String;
}

impl StringFallback for String {
    fn or_else_if_empty<F>(self, fallback: F) -> Option<String>
    where
        F: FnOnce() -> String,
    {
        if self.is_empty() {
            let alt = fallback();
            (!alt.is_empty()).then_some(alt)
        } else {
            Some(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_zero_remote_command_uses_action_label_not_shell_source() {
        let err = remote_command_failed(
            "media-node",
            "docker.inspect",
            "docker inspect radarr --format '{{json .Config.Env}}' SECRET=abc123",
            RemoteCommandOutput {
                stdout: String::new(),
                stderr: "permission denied".to_owned(),
                exit_status: Some(1),
            },
        );

        let rendered = err.to_string();
        assert!(rendered.contains("docker.inspect"));
        assert!(!rendered.contains("SECRET=abc123"));
        assert!(!rendered.contains("docker inspect radarr"));
    }

    #[test]
    fn missing_exit_status_is_reported_as_failure() {
        let err = remote_command_failed(
            "media-node",
            "docker.ps",
            "docker ps",
            RemoteCommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_status: None,
            },
        );

        let ExtractError::RemoteCommand {
            exit_status,
            message,
            ..
        } = err
        else {
            panic!("expected remote command error");
        };

        assert_eq!(exit_status, None);
        assert_eq!(message, "remote command did not report an exit status");
    }
}
