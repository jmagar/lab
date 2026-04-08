//! File-reading transports: local fs and ssh.
//!
//! Uses **enum dispatch** instead of `dyn Transport` because the set of
//! transports is closed (two variants), and enum dispatch keeps the public
//! API object-safety-friendly without `async-trait`.

use std::path::{Path, PathBuf};

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
    async fn read(&self, _path: &Path) -> Result<Vec<u8>, ExtractError> {
        // Real impl: tokio::fs::read(path).await.map_err(|e| ExtractError::Io { ... })
        Err(ExtractError::Io {
            path: _path.to_path_buf(),
            source: std::io::Error::other("LocalFs::read not yet implemented"),
        })
    }

    async fn list_subdirs(&self, _dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        // Real impl: tokio::fs::read_dir + filter is_dir
        Err(ExtractError::Io {
            path: _dir.to_path_buf(),
            source: std::io::Error::other("LocalFs::list_subdirs not yet implemented"),
        })
    }
}

/// SSH transport — opens a `russh` session to a host alias resolved via
/// `~/.ssh/config`, runs an SFTP subsystem, and reads files over the channel.
#[derive(Debug)]
pub struct SshFs {
    /// SSH host alias as it appears in `~/.ssh/config`.
    pub host: String,
}

impl SshFs {
    /// Open a new SSH connection. Authentication is whatever ssh-agent has
    /// loaded plus any keys/certs/jump-hosts declared in `~/.ssh/config`.
    ///
    /// # Errors
    /// Returns `ExtractError::Ssh` on connection or auth failure.
    pub async fn connect(host: impl Into<String>) -> Result<Self, ExtractError> {
        // Real impl: russh::client::connect_stream + agent auth + open sftp subsystem
        Ok(Self { host: host.into() })
    }

    async fn read(&self, _path: &Path) -> Result<Vec<u8>, ExtractError> {
        Err(ExtractError::Ssh {
            host: self.host.clone(),
            message: "SshFs::read not yet implemented".to_owned(),
        })
    }

    async fn list_subdirs(&self, _dir: &Path) -> Result<Vec<PathBuf>, ExtractError> {
        Err(ExtractError::Ssh {
            host: self.host.clone(),
            message: "SshFs::list_subdirs not yet implemented".to_owned(),
        })
    }
}
