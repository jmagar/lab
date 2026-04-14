use serde::{Deserialize, Serialize};

use crate::error::AuthError;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub subject: Option<String>,
}

pub fn validate_access_token(token: &str) -> Result<Claims, AuthError> {
    let _ = token;
    Err(AuthError::UnconfiguredVerifier)
}
