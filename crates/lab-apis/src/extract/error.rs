//! `ExtractError` — typed errors for the extract module.

use std::path::PathBuf;

use thiserror::Error;

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
    #[error("no recognized service configs found under {path}")]
    NothingFound {
        /// The appdata root that was scanned.
        path: PathBuf,
    },
}
