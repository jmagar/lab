use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("invalid access token")]
    InvalidAccessToken,

    #[error("access token verifier is not configured")]
    UnconfiguredVerifier,
}
