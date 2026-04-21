use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum HmacError {
    #[error("signature header missing 'sha256=' prefix")]
    MissingPrefix,
    #[error("signature is not valid hex")]
    BadHex,
    #[error("signature length mismatch")]
    BadLength,
    #[error("signature does not match")]
    Mismatch,
}

pub fn verify_signature(secret: &[u8], body: &[u8], header: &str) -> Result<(), HmacError> {
    let hex_part = header.strip_prefix("sha256=").ok_or(HmacError::MissingPrefix)?;
    let provided = hex::decode(hex_part).map_err(|_| HmacError::BadHex)?;
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    let computed = mac.finalize().into_bytes();
    if provided.len() != computed.len() {
        return Err(HmacError::BadLength);
    }
    if provided.ct_eq(&computed).into() {
        Ok(())
    } else {
        Err(HmacError::Mismatch)
    }
}
