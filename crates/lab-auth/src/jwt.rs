use std::fmt;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::rand_core::{TryCryptoRng, TryRng, UnwrapErr};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AuthError;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    pub scope: String,
    pub azp: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwksDocument {
    pub keys: Vec<JwkKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JwkKey {
    pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Clone)]
pub struct SigningKeys {
    pub key_id: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    jwks: JwksDocument,
}

impl fmt::Debug for SigningKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SigningKeys")
            .field("key_id", &self.key_id)
            .finish_non_exhaustive()
    }
}

pub fn validate_access_token(token: &str) -> Result<AccessClaims, AuthError> {
    let _ = token;
    Err(AuthError::UnconfiguredVerifier)
}

impl SigningKeys {
    pub fn load_or_create(path: PathBuf) -> Result<Self, AuthError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                AuthError::Storage(format!(
                    "create signing key directory `{}`: {error}",
                    parent.display()
                ))
            })?;
        }

        let existed = path.exists();
        if existed {
            ensure_restrictive_permissions(&path)?;
        }

        let private_key = if existed {
            let pem = std::fs::read_to_string(&path).map_err(|error| {
                AuthError::Storage(format!("read signing key `{}`: {error}", path.display()))
            })?;
            RsaPrivateKey::from_pkcs8_pem(&pem)
                .map_err(|error| AuthError::Storage(format!("parse signing key PEM: {error}")))?
        } else {
            let mut rng = UnwrapErr(SystemRng);
            let key = RsaPrivateKey::new(&mut rng, 2048)
                .map_err(|error| AuthError::Storage(format!("generate RSA signing key: {error}")))?;
            let pem = key
                .to_pkcs8_pem(LineEnding::LF)
                .map_err(|error| AuthError::Storage(format!("encode signing key PEM: {error}")))?;
            std::fs::write(&path, pem.as_bytes()).map_err(|error| {
                AuthError::Storage(format!("write signing key `{}`: {error}", path.display()))
            })?;
            set_restrictive_permissions(&path)?;
            key
        };

        ensure_restrictive_permissions(&path)?;
        Self::from_private_key(private_key)
    }

    pub fn issue_access_token(&self, claims: AccessClaims) -> Result<String, AuthError> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.key_id.clone());
        encode(&header, &claims, &self.encoding_key)
            .map_err(|error| AuthError::Storage(format!("encode access token: {error}")))
    }

    pub fn validate_access_token(&self, token: &str) -> Result<AccessClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        decode::<AccessClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidAccessToken)
    }

    pub fn jwks(&self) -> &JwksDocument {
        &self.jwks
    }

    fn from_private_key(private_key: RsaPrivateKey) -> Result<Self, AuthError> {
        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|error| AuthError::Storage(format!("encode signing key PEM: {error}")))?;
        let public_key = RsaPublicKey::from(&private_key);
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|error| AuthError::Storage(format!("encode public key PEM: {error}")))?;
        let public_der = public_key
            .to_public_key_der()
            .map_err(|error| AuthError::Storage(format!("encode public key DER: {error}")))?;
        let digest = Sha256::digest(public_der.as_bytes());
        let key_id = URL_SAFE_NO_PAD.encode(&digest[..12]);

        let jwks = JwksDocument {
            keys: vec![JwkKey {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                alg: "RS256".to_string(),
                kid: key_id.clone(),
                n: URL_SAFE_NO_PAD.encode(public_key.n_bytes()),
                e: URL_SAFE_NO_PAD.encode(public_key.e_bytes()),
            }],
        };

        Ok(Self {
            key_id,
            encoding_key: EncodingKey::from_rsa_pem(private_pem.as_bytes()).map_err(|error| {
                AuthError::Storage(format!("load RSA encoding key from PEM: {error}"))
            })?,
            decoding_key: DecodingKey::from_rsa_pem(public_pem.as_bytes()).map_err(|error| {
                AuthError::Storage(format!("load RSA decoding key from PEM: {error}"))
            })?,
            jwks,
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct SystemRng;

impl TryRng for SystemRng {
    type Error = SystemRngError;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut bytes = [0_u8; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut bytes = [0_u8; 8];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        getrandom::fill(dst).map_err(SystemRngError)
    }
}

impl TryCryptoRng for SystemRng {}

#[derive(Debug)]
struct SystemRngError(getrandom::Error);

impl fmt::Display for SystemRngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for SystemRngError {}

#[cfg(unix)]
fn ensure_restrictive_permissions(path: &Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)
        .map_err(|error| AuthError::Storage(format!("stat `{}`: {error}", path.display())))?;
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(AuthError::InsecurePermissions {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_restrictive_permissions(_path: &Path) -> Result<(), AuthError> {
    Ok(())
}

#[cfg(unix)]
fn set_restrictive_permissions(path: &Path) -> Result<(), AuthError> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(|error| {
        AuthError::Storage(format!("chmod 0600 `{}`: {error}", path.display()))
    })
}

#[cfg(not(unix))]
fn set_restrictive_permissions(_path: &Path) -> Result<(), AuthError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::decode_header;

    use super::{AccessClaims, SigningKeys};

    #[test]
    fn generated_key_is_reused_on_second_load() {
        let dir = tempfile::tempdir().unwrap();
        let first = SigningKeys::load_or_create(dir.path().join("auth-jwt.pem")).unwrap();
        let second = SigningKeys::load_or_create(dir.path().join("auth-jwt.pem")).unwrap();
        assert_eq!(first.key_id, second.key_id);
    }

    #[cfg(unix)]
    #[test]
    fn signing_key_refuses_world_readable_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth-jwt.pem");
        std::fs::write(&path, "bad").unwrap();
        std::fs::set_permissions(&path, PermissionsExt::from_mode(0o644)).unwrap();
        let err = SigningKeys::load_or_create(path).unwrap_err();
        assert!(err.to_string().contains("permissions"));
    }

    #[test]
    fn minted_access_token_round_trips_and_contains_kid() {
        let signer = test_signer();
        let token = signer.issue_access_token(sample_claims()).unwrap();
        let claims = signer.validate_access_token(&token).unwrap();
        assert_eq!(claims.aud, "https://lab.example.com");
        assert!(!claims.jti.is_empty());
        assert!(decode_header(&token).unwrap().kid.is_some());
    }

    fn test_signer() -> SigningKeys {
        let dir = tempfile::tempdir().unwrap();
        SigningKeys::load_or_create(dir.path().join("auth-jwt.pem")).unwrap()
    }

    fn sample_claims() -> AccessClaims {
        AccessClaims {
            iss: "https://lab.example.com".to_string(),
            sub: "google-user".to_string(),
            aud: "https://lab.example.com".to_string(),
            exp: 4_102_444_800,
            iat: 1_700_000_000,
            jti: "test-jti".to_string(),
            scope: "lab".to_string(),
            azp: "client".to_string(),
        }
    }
}
