//! Beads error type.

#[derive(Debug, thiserror::Error)]
pub enum BeadsError {
    #[error("bd command failed ({command}): {message}")]
    Command { command: String, message: String },

    #[error("bd command output was not valid JSON ({command}): {message}")]
    Decode { command: String, message: String },
}
