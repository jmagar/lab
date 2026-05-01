use lab_apis::beads::BeadsClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn client_from_env() -> Option<BeadsClient> {
    Some(BeadsClient::new(
        env_non_empty("BEADS_BIN").unwrap_or_else(|| "bd".to_owned()),
        None,
    ))
}

pub fn require_client() -> Result<BeadsClient, ToolError> {
    Ok(client_from_env().unwrap_or_default())
}

pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "Beads local CLI contract is unavailable".into(),
    }
}
