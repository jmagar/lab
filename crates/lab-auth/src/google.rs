use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{Algorithm, DecodingKey, Header, Validation, decode, decode_header};
use reqwest::Url;
use reqwest::header;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::AuthError;
use crate::util::fingerprint;

const GOOGLE_AUTHORIZE_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_JWKS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUER: &str = "https://accounts.google.com";
const GOOGLE_HTTP_TIMEOUT: Duration = Duration::from_secs(30);
const GOOGLE_DEFAULT_JWKS_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizeUrlRequest {
    pub state: String,
    pub scope: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Clone)]
pub struct GoogleProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Url,
    pub scopes: Vec<String>,
    pub http: reqwest::Client,
    authorize_endpoint: Url,
    token_endpoint: Url,
    jwks_endpoint: Url,
    jwks_cache: Arc<RwLock<Option<CachedGoogleJwks>>>,
}

impl std::fmt::Debug for GoogleProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleProvider")
            .field("client_id", &self.client_id)
            .field("redirect_uri", &self.redirect_uri)
            .field("scopes", &self.scopes)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleExchange {
    pub subject: String,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct GoogleIdTokenClaims {
    iss: String,
    sub: String,
    #[serde(default)]
    email: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct GoogleJwks {
    keys: Vec<GoogleJwk>,
}

#[derive(Clone, Debug, Deserialize)]
struct GoogleJwk {
    kid: String,
    #[serde(default)]
    alg: Option<String>,
    n: String,
    e: String,
}

#[derive(Clone, Debug)]
struct CachedGoogleJwks {
    jwks: GoogleJwks,
    expires_at: Instant,
}

struct GoogleRequestTrace<'a> {
    operation: &'static str,
    method: &'static str,
    endpoint: &'a Url,
    started: Instant,
}

impl<'a> GoogleRequestTrace<'a> {
    fn start(operation: &'static str, method: &'static str, endpoint: &'a Url) -> Self {
        info!(
            provider = "google",
            operation,
            method,
            host = endpoint.host_str().unwrap_or_default(),
            path = endpoint.path(),
            "request.start"
        );
        Self {
            operation,
            method,
            endpoint,
            started: Instant::now(),
        }
    }

    fn finish(&self, status: reqwest::StatusCode) {
        info!(
            provider = "google",
            operation = self.operation,
            method = self.method,
            host = self.endpoint.host_str().unwrap_or_default(),
            path = self.endpoint.path(),
            status = status.as_u16(),
            elapsed_ms = self.started.elapsed().as_millis(),
            "request.finish"
        );
    }

    fn error(&self, status: Option<reqwest::StatusCode>, error: &reqwest::Error) {
        match status {
            Some(status) => warn!(
                provider = "google",
                operation = self.operation,
                method = self.method,
                host = self.endpoint.host_str().unwrap_or_default(),
                path = self.endpoint.path(),
                status = status.as_u16(),
                elapsed_ms = self.started.elapsed().as_millis(),
                error = %error,
                "request.error"
            ),
            None => warn!(
                provider = "google",
                operation = self.operation,
                method = self.method,
                host = self.endpoint.host_str().unwrap_or_default(),
                path = self.endpoint.path(),
                elapsed_ms = self.started.elapsed().as_millis(),
                error = %error,
                "request.error"
            ),
        }
    }
}

struct GoogleRequestErrors {
    transport_context: &'static str,
    transport_log: &'static str,
    status_context: &'static str,
    status_log: &'static str,
    decode_context: &'static str,
    decode_log: &'static str,
}

async fn read_json_response<T: DeserializeOwned>(
    trace: GoogleRequestTrace<'_>,
    request: reqwest::RequestBuilder,
    errors: GoogleRequestErrors,
) -> Result<T, AuthError> {
    let response = request.send().await.map_err(|error| {
        trace.error(None, &error);
        warn!(provider = "google", error = %error, "{}", errors.transport_log);
        AuthError::Storage(format!("{}: {error}", errors.transport_context))
    })?;
    let status = response.status();
    let response = response.error_for_status().map_err(|error| {
        trace.error(Some(status), &error);
        warn!(provider = "google", error = %error, "{}", errors.status_log);
        AuthError::Storage(format!("{}: {error}", errors.status_context))
    })?;
    trace.finish(status);
    response.json::<T>().await.map_err(|error| {
        warn!(provider = "google", error = %error, "{}", errors.decode_log);
        AuthError::Storage(format!("{}: {error}", errors.decode_context))
    })
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: Url) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            http: reqwest::Client::builder()
                .timeout(GOOGLE_HTTP_TIMEOUT)
                .build()
                .expect("google oauth http client should build"),
            authorize_endpoint: Url::parse(GOOGLE_AUTHORIZE_ENDPOINT)
                .expect("valid google auth url"),
            token_endpoint: Url::parse(GOOGLE_TOKEN_ENDPOINT).expect("valid google token url"),
            jwks_endpoint: Url::parse(GOOGLE_JWKS_ENDPOINT).expect("valid google jwks url"),
            jwks_cache: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg(test)]
    pub fn with_endpoints(mut self, authorize_endpoint: Url, token_endpoint: Url) -> Self {
        self.authorize_endpoint = authorize_endpoint;
        self.token_endpoint = token_endpoint;
        self
    }

    #[cfg(test)]
    pub fn with_jwks_endpoint(mut self, jwks_endpoint: Url) -> Self {
        self.jwks_endpoint = jwks_endpoint;
        self
    }

    pub fn authorize_url(&self, request: AuthorizeUrlRequest) -> Result<Url, AuthError> {
        let mut url = self.authorize_endpoint.clone();
        let scope = self.scopes.join(" ");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", self.redirect_uri.as_str())
            .append_pair("response_type", "code")
            .append_pair("scope", &scope)
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent")
            .append_pair("include_granted_scopes", "true")
            .append_pair("state", &request.state)
            .append_pair("code_challenge", &request.code_challenge)
            .append_pair("code_challenge_method", &request.code_challenge_method);
        debug!(
            provider = "google",
            oauth_state_id = %fingerprint(&request.state),
            scope = %scope,
            redirect_uri = %self.redirect_uri,
            "oauth upstream authorize URL constructed"
        );
        Ok(url)
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<GoogleExchange, AuthError> {
        let trace = GoogleRequestTrace::start("code_exchange", "POST", &self.token_endpoint);
        info!(
            provider = "google",
            oauth_code_id = %fingerprint(code),
            redirect_uri = %self.redirect_uri,
            "oauth upstream code exchange started"
        );
        let payload: GoogleTokenResponse = read_json_response(
            trace,
            self.http.post(self.token_endpoint.clone()).form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("code_verifier", code_verifier),
            ]),
            GoogleRequestErrors {
                transport_context: "exchange google auth code",
                transport_log: "oauth upstream code exchange request failed",
                status_context: "google token endpoint error",
                status_log: "oauth upstream code exchange returned error status",
                decode_context: "decode google token response",
                decode_log: "oauth upstream code exchange returned an unreadable payload",
            },
        )
        .await?;
        let claims = self.verify_id_token(&payload.id_token).await?;
        info!(
            provider = "google",
            subject_id = %fingerprint(&claims.sub),
            has_refresh_token = payload.refresh_token.is_some(),
            expires_in_secs = payload.expires_in,
            "oauth upstream code exchange succeeded"
        );
        Ok(GoogleExchange {
            subject: claims.sub,
            email: claims.email,
            access_token: payload.access_token,
            refresh_token: payload.refresh_token,
            expires_in: payload.expires_in,
            id_token: payload.id_token,
        })
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<GoogleExchange, AuthError> {
        let trace = GoogleRequestTrace::start("refresh", "POST", &self.token_endpoint);
        info!(
            provider = "google",
            refresh_token_id = %fingerprint(refresh_token),
            "oauth upstream refresh started"
        );
        let payload: GoogleTokenResponse = read_json_response(
            trace,
            self.http.post(self.token_endpoint.clone()).form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ]),
            GoogleRequestErrors {
                transport_context: "refresh google token",
                transport_log: "oauth upstream refresh request failed",
                status_context: "google refresh endpoint error",
                status_log: "oauth upstream refresh returned error status",
                decode_context: "decode google refresh response",
                decode_log: "oauth upstream refresh returned an unreadable payload",
            },
        )
        .await?;
        let claims = self.verify_id_token(&payload.id_token).await?;
        info!(
            provider = "google",
            subject_id = %fingerprint(&claims.sub),
            has_refresh_token = payload.refresh_token.is_some(),
            expires_in_secs = payload.expires_in,
            "oauth upstream refresh succeeded"
        );
        Ok(GoogleExchange {
            subject: claims.sub,
            email: claims.email,
            access_token: payload.access_token,
            refresh_token: payload.refresh_token,
            expires_in: payload.expires_in,
            id_token: payload.id_token,
        })
    }

    async fn verify_id_token(&self, id_token: &str) -> Result<GoogleIdTokenClaims, AuthError> {
        let header = decode_header(id_token)
            .map_err(|error| AuthError::Storage(format!("verify google id_token: {error}")))?;
        validate_id_token_header(&header)?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Storage("google id_token is missing a key id".to_string()))?;
        let jwks = self.fetch_jwks().await?;
        let key = jwks
            .keys
            .into_iter()
            .find(|key| key.kid == kid)
            .ok_or_else(|| {
                AuthError::Storage("google id_token key id was not found in JWKS".to_string())
            })?;
        if let Some(alg) = key.alg.as_deref()
            && alg != "RS256"
        {
            return Err(AuthError::Storage(format!(
                "google JWKS key `{}` uses unsupported algorithm `{alg}`",
                key.kid
            )));
        }

        let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e).map_err(|error| {
            AuthError::Storage(format!("build google id_token decoding key: {error}"))
        })?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.leeway = 0;
        validation.set_audience(&[self.client_id.as_str()]);

        let claims = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|error| AuthError::Storage(format!("invalid google id_token: {error}")))?;

        if claims.iss != GOOGLE_ISSUER && claims.iss != "accounts.google.com" {
            return Err(AuthError::Storage(format!(
                "invalid google id_token issuer `{}`",
                claims.iss
            )));
        }

        Ok(claims)
    }

    async fn fetch_jwks(&self) -> Result<GoogleJwks, AuthError> {
        if let Some(jwks) = self.cached_jwks().await {
            debug!(provider = "google", "google jwks cache hit");
            return Ok(jwks);
        }

        let mut cache = self.jwks_cache.write().await;
        if let Some(cached) = cache.as_ref()
            && cached.expires_at > Instant::now()
        {
            debug!(
                provider = "google",
                "google jwks cache hit after refresh lock"
            );
            return Ok(cached.jwks.clone());
        }

        let trace = GoogleRequestTrace::start("fetch_jwks", "GET", &self.jwks_endpoint);
        let response = self
            .http
            .get(self.jwks_endpoint.clone())
            .send()
            .await
            .map_err(|error| {
                trace.error(None, &error);
                warn!(provider = "google", error = %error, "google jwks request failed");
                AuthError::Storage(format!("fetch google jwks: {error}"))
            })?;
        let status = response.status();
        let ttl = google_jwks_ttl(response.headers());
        let response = response.error_for_status().map_err(|error| {
            trace.error(Some(status), &error);
            warn!(provider = "google", error = %error, "google jwks request returned error status");
            AuthError::Storage(format!("google jwks endpoint error: {error}"))
        })?;
        trace.finish(status);
        let jwks = response.json::<GoogleJwks>().await.map_err(|error| {
            warn!(provider = "google", error = %error, "google jwks payload unreadable");
            AuthError::Storage(format!("decode google jwks response: {error}"))
        })?;

        *cache = Some(CachedGoogleJwks {
            jwks: jwks.clone(),
            expires_at: Instant::now() + ttl,
        });

        Ok(jwks)
    }

    async fn cached_jwks(&self) -> Option<GoogleJwks> {
        let cache = self.jwks_cache.read().await;
        cache
            .as_ref()
            .filter(|cached| cached.expires_at > Instant::now())
            .map(|cached| cached.jwks.clone())
    }
}

fn google_jwks_ttl(headers: &header::HeaderMap) -> Duration {
    headers
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_max_age)
        .map(Duration::from_secs)
        .unwrap_or(GOOGLE_DEFAULT_JWKS_TTL)
}

fn parse_max_age(cache_control: &str) -> Option<u64> {
    cache_control.split(',').find_map(|directive| {
        let directive = directive.trim();
        let value = directive.strip_prefix("max-age=")?;
        value.parse::<u64>().ok()
    })
}

fn validate_id_token_header(header: &Header) -> Result<(), AuthError> {
    if header.alg != Algorithm::RS256 {
        return Err(AuthError::Storage(format!(
            "verify google id_token: unsupported algorithm `{:?}`",
            header.alg
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use rsa::RsaPrivateKey;
    use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
    use rsa::traits::PublicKeyParts;
    use serde_json::json;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{AuthorizeUrlRequest, GoogleProvider};

    #[test]
    fn google_authorize_url_includes_offline_access_prompt_and_pkce() {
        let provider = test_google_provider();
        let url = provider.authorize_url(sample_request()).unwrap();
        assert!(url.as_str().contains("access_type=offline"));
        assert!(url.as_str().contains("prompt=consent"));
        assert!(url.as_str().contains("code_challenge="));
    }

    #[tokio::test]
    async fn google_exchange_parses_subject_and_refresh_token() {
        let provider = mocked_google_provider().await;
        let token = provider.exchange_code("code", "verifier").await.unwrap();
        assert_eq!(token.subject, "google-subject-123");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-token"));
    }

    #[tokio::test]
    async fn google_exchange_rejects_unsigned_id_tokens() {
        let provider = mocked_google_provider_with_id_token(test_id_token()).await;
        let error = provider
            .exchange_code("code", "verifier")
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("verify google id_token"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn google_exchange_rejects_wrong_audience_in_id_token() {
        let provider =
            mocked_google_provider_with_id_token(signed_test_id_token("other-client", false, true))
                .await;
        let error = provider
            .exchange_code("code", "verifier")
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("invalid google id_token"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn google_exchange_rejects_expired_id_token() {
        let provider =
            mocked_google_provider_with_id_token(signed_test_id_token("client-id", true, true))
                .await;
        let error = provider
            .exchange_code("code", "verifier")
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("invalid google id_token"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn google_exchange_rejects_wrong_issuer_in_id_token() {
        let provider =
            mocked_google_provider_with_id_token(signed_test_id_token("client-id", false, false))
                .await;
        let error = provider
            .exchange_code("code", "verifier")
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("issuer"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn google_exchange_reuses_cached_jwks() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "google-access-token",
                "refresh_token": "refresh-token",
                "expires_in": 3600,
                "id_token": signed_test_id_token("client-id", false, true),
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Cache-Control", "public, max-age=3600")
                    .set_body_json(test_jwks()),
            )
            .mount(&server)
            .await;

        let provider = test_google_provider()
            .with_endpoints(
                server.uri().parse::<Url>().unwrap(),
                server.uri().parse::<Url>().unwrap().join("/token").unwrap(),
            )
            .with_jwks_endpoint(server.uri().parse::<Url>().unwrap().join("/certs").unwrap());

        provider.exchange_code("code-1", "verifier").await.unwrap();
        provider.exchange_code("code-2", "verifier").await.unwrap();

        let requests = server.received_requests().await.unwrap();
        let jwks_requests = requests
            .iter()
            .filter(|request| request.url.path() == "/certs")
            .count();
        assert_eq!(jwks_requests, 1);
    }

    #[test]
    fn parse_max_age_reads_cache_control_max_age() {
        assert_eq!(super::parse_max_age("public, max-age=3600"), Some(3600));
        assert_eq!(super::parse_max_age("no-cache"), None);
    }

    fn test_google_provider() -> GoogleProvider {
        GoogleProvider::new(
            "client-id".to_string(),
            "client-secret".to_string(),
            Url::parse("https://lab.example.com/auth/google/callback").unwrap(),
        )
    }

    async fn mocked_google_provider() -> GoogleProvider {
        mocked_google_provider_with_id_token(signed_test_id_token("client-id", false, true)).await
    }

    async fn mocked_google_provider_with_id_token(id_token: String) -> GoogleProvider {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "google-access-token",
                "refresh_token": "refresh-token",
                "expires_in": 3600,
                "id_token": id_token,
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(test_jwks()))
            .mount(&server)
            .await;

        test_google_provider()
            .with_endpoints(
                server.uri().parse::<Url>().unwrap(),
                server.uri().parse::<Url>().unwrap().join("/token").unwrap(),
            )
            .with_jwks_endpoint(server.uri().parse::<Url>().unwrap().join("/certs").unwrap())
    }

    fn sample_request() -> AuthorizeUrlRequest {
        AuthorizeUrlRequest {
            state: "state-123".to_string(),
            scope: "lab".to_string(),
            code_challenge: "challenge".to_string(),
            code_challenge_method: "S256".to_string(),
        }
    }

    fn test_id_token() -> String {
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"google-subject-123"}"#);
        format!("{header}.{payload}.")
    }

    fn signed_test_id_token(client_id: &str, expired: bool, valid_issuer: bool) -> String {
        let claims = json!({
            "iss": if valid_issuer { "https://accounts.google.com" } else { "https://evil.example.com" },
            "aud": client_id,
            "sub": "google-subject-123",
            "email": "user@example.com",
            "iat": (unix_now() - 10) as usize,
            "exp": if expired { (unix_now() - 3600) as usize } else { (unix_now() + 3600) as usize },
        });
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("test-kid".to_string());
        encode(&header, &claims, &test_encoding_key()).unwrap()
    }

    fn test_jwks() -> serde_json::Value {
        let key = test_rsa_key();
        let public_key = key.to_public_key();
        json!({
            "keys": [{
                "kid": "test-kid",
                "alg": "RS256",
                "kty": "RSA",
                "use": "sig",
                "n": URL_SAFE_NO_PAD.encode(public_key.n_bytes()),
                "e": URL_SAFE_NO_PAD.encode(public_key.e_bytes()),
            }]
        })
    }

    fn test_rsa_key() -> RsaPrivateKey {
        RsaPrivateKey::from_pkcs8_pem(TEST_RSA_KEY_PEM).unwrap()
    }

    fn test_encoding_key() -> EncodingKey {
        let pem = test_rsa_key().to_pkcs8_pem(Default::default()).unwrap();
        EncodingKey::from_rsa_pem(pem.as_bytes()).unwrap()
    }

    fn unix_now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    const TEST_RSA_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC/Wa3MQnrNbKu9
H5+ZH30lrKV3+EJeuY0ofx3qMx73ax+ArHaPFHXq3PUAalSZ+UlBqRmX89DdzwWG
l5hqt3wzGjGe49zxhY5+nUUPLtRiI4JH0iEH4Bg3W9e9gWAAPjVemuYmZ57R9XOd
O1l0aI20mZiy4jeEN7Ls40I/pwyTcB22krOeHz13E1NzG+uDQnaMZkOKomRdTkKr
tiSETBcpacpIdyLtdc9lHR4LbcZtBH3aMosjmgae3uvQyks6ntj0UQZaKNYqNwNE
+GSOqQdtJeoWhps1IYjhc9wcfrlL69nn5U4FXwCcPzGOKXCOW45/BB4nr2WF2Bkq
N7iytDv/AgMBAAECggEABt1BtdUgsKPYWVV8FTMi+yoBWZdnUhyX6r78pL0mvDt0
itok+qcCP+WjSFuII2nk7d0SFPhjIsHdceGYTyO76d1jsE5+S4+9997ObmgAqHCb
qNXp521rkPjTeXHdrsSMh5NI9FG9SczjU92gLOPfSX5FEw24bh7NZWAVrVDhy5wn
BWAZow2kByQ2SLRitUJr+a1xF3UO3PgHLKdP0H0qZp9TCar3nzJxwMUyGJxOcd4f
mElyYNIsJtOBsIIoBsNh+aj5pSjOiuEZmfipbHuMWpjEwF1+UVH4iPXQugyKgFze
Gc8wy3aFlmA4dH2jbSzP3aIwiFUDgqsUrqdyEXVVeQKBgQD5/psH3uk3AOkRC/k/
P6cI5pwFG0rFRe3UgBJFqODnbTZR+0BwyTqf9kCZgi0nJIudCNyUF5utl8rkWdwE
s2s42NibGWTVyb5dabT+dHwP42jFljCxxbZw1D3GmP1mX0ybyXj0BOqWEpMHc76q
ZxzJFfML0FfyTxMVycukBL4bEwKBgQDD8m2Y5GvO17RJDeG6yPupTvWbcBaUTuwe
0w9LOWSOYi3YPAIt7m6yE9XH9cWSFqXMoOAS5Lu1zUuBvwhZz3XAAeL9JpU2F/1V
DW7NiChNb7Np2X1dUHZTS5EmaAkok55uEMfA1N1FhsDfN+qCxVPITUszYwrPCu52
SMd4Nx5s5QKBgQDfK6woTZWyNYzaW+8IyIEL0BqN8HxCOZgD8MTfDNChqHwqmXpA
dVNxg3rNz0kRvW0pJcUMKzsdr/k++v0P8T+RwvszEmtS8sOPTpN16HTsFh3s7ZPQ
z2h7tuzjAqaMIh0YobXpWQ42JKS+rVQTePNYi9CpxjcMqAyokbnKVTWEowKBgFrB
5/eAHVsh19RahKoyOzZRZztGsH6jC4S/d379J1E3skpMiSnjHQyIWWWTtZ4TtVnR
TdgSb8smOonvBJwsljqH5S4h98ylUeZaIW87WId9bFljrkhRY2zzPFjQqSVNMn2C
cjMjpRV189GwIYPOiB7nhiRYBIKfapII5bMNvJ7tAoGACMvtonFh25b7gB7j3Pep
LEH/fA5CRiOs7Plrt2Sv54wAup4Y6+HQ8i/KFOXIejEN9vfY1YRfyD5Ajc05zg90
uE8aLb5YtFvoaLAnc/A2ceW8sNxGgT5aPyLPUdmfSryAO4ayFDHmRlGFRsZtTUbn
Iy60nwnOxK6B5mZV2Cs+kv8=
-----END PRIVATE KEY-----"#;
}
