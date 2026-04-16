//! `ExtractError` — typed errors for the extract module.

use std::path::PathBuf;

use thiserror::Error;

use super::types::ScanTarget;

/// Errors returned by `ExtractClient`.
#[derive(Debug, Error)]
pub enum ExtractError {
    /// The URI couldn't be parsed.
    #[error("invalid uri '{input}': {reason}")]
    InvalidUri {
        /// The raw input string.
        input: String,
        /// Why parsing failed.
        reason: &'static str,
    },

    /// I/O error reading a file (local or remote).
    #[error("io error reading {path}: {source}")]
    Io {
        /// The path being read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// SSH transport error (connection, auth, channel).
    #[error("ssh error against {host}: {message}")]
    Ssh {
        /// SSH host alias.
        host: String,
        /// Underlying error message.
        message: String,
    },

    /// A typed remote inspection command failed.
    #[error("remote command '{action}' failed on {host}: {message}")]
    RemoteCommand {
        /// SSH host alias.
        host: String,
        /// Typed action label, not the raw shell command.
        action: String,
        /// Exit status when the remote process reported one.
        exit_status: Option<u32>,
        /// Sanitized stderr/stdout summary.
        message: String,
    },

    /// A parser couldn't extract creds from a file it found.
    #[error("parse error in {service} config at {path}: {message}")]
    Parse {
        /// Service whose parser failed.
        service: String,
        /// Path to the file being parsed.
        path: PathBuf,
        /// Why parsing failed.
        message: String,
    },

    /// No appdata subdirectories matched any known parser.
    #[error("{}", nothing_found_message(target))]
    NothingFound {
        /// The scan target that produced no recognized services.
        target: ScanTarget,
    },
}

fn nothing_found_message(target: &ScanTarget) -> String {
    match target {
        ScanTarget::Targeted(uri) => {
            format!(
                "no recognized service configs found under {}",
                uri.path().display()
            )
        }
        ScanTarget::Fleet => "no recognized service configs found during fleet scan".to_owned(),
    }
}
