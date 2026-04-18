//! Shared types for outbound upstream OAuth.

use thiserror::Error;

/// Stable error kinds for upstream OAuth flows.
///
/// These must be kept in sync with `docs/ERRORS.md`.
#[derive(Debug, Error)]
pub enum OauthError {
    /// Refresh token was rejected (`invalid_grant`) or decryption failed after key
    /// rotation.  User must re-initiate the authorization flow.
    #[error("oauth_needs_reauth: {0}")]
    NeedsReauth(String),

    /// Callback state is missing, expired, replayed, or bound to a different
    /// subject / upstream.
    #[error("oauth_state_invalid: {0}")]
    StateInvalid(String),

    /// Upstream AS refused the `resource` parameter or issued a token with the
    /// wrong audience (RFC 8707).
    #[error("oauth_resource_mismatch: {0}")]
    ResourceMismatch(String),

    /// AS metadata `issuer` did not match the discovered AS URL (RFC 8414 §3.3).
    #[error("oauth_issuer_mismatch: {0}")]
    IssuerMismatch(String),

    /// AS only offered `plain` PKCE or omitted `code_challenge_methods_supported`.
    #[error("oauth_unsupported_method: {0}")]
    UnsupportedMethod(String),

    /// Internal / configuration errors that are not caller-recoverable.
    #[error("upstream_oauth internal error: {0}")]
    Internal(String),
}

impl OauthError {
    /// Stable `kind` string for structured log / envelope fields.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NeedsReauth(_) => "oauth_needs_reauth",
            Self::StateInvalid(_) => "oauth_state_invalid",
            Self::ResourceMismatch(_) => "oauth_resource_mismatch",
            Self::IssuerMismatch(_) => "oauth_issuer_mismatch",
            Self::UnsupportedMethod(_) => "oauth_unsupported_method",
            Self::Internal(_) => "internal_error",
        }
    }
}

/// Return value of [`manager::UpstreamOauthManager::begin_authorization`].
#[derive(Debug)]
pub struct BeginAuthorization {
    /// URL the operator's browser must navigate to.
    pub authorization_url: String,
}
