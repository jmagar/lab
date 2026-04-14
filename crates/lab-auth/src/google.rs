use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::error::AuthError;

const GOOGLE_AUTHORIZE_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

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
    sub: String,
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: Url) -> Result<Self, AuthError> {
        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            http: reqwest::Client::new(),
            authorize_endpoint: Url::parse(GOOGLE_AUTHORIZE_ENDPOINT)
                .map_err(|e| AuthError::Config(format!("invalid Google authorize endpoint: {e}")))?,
            token_endpoint: Url::parse(GOOGLE_TOKEN_ENDPOINT)
                .map_err(|e| AuthError::Config(format!("invalid Google token endpoint: {e}")))?,
        })
    }

    #[cfg(test)]
    pub fn with_endpoints(mut self, authorize_endpoint: Url, token_endpoint: Url) -> Self {
        self.authorize_endpoint = authorize_endpoint;
        self.token_endpoint = token_endpoint;
        self
    }

    pub fn authorize_url(&self, request: &AuthorizeUrlRequest) -> Result<Url, AuthError> {
        let mut url = self.authorize_endpoint.clone();
        let scope = self.scopes.join(" ");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", self.redirect_uri.as_str())
            .append_pair("response_type", "code")
            .append_pair("scope", &scope)
            .append_pair("access_type", "offline")
            .append_pair("include_granted_scopes", "true")
            .append_pair("state", &request.state)
            .append_pair("code_challenge", &request.code_challenge)
            .append_pair("code_challenge_method", &request.code_challenge_method);
        Ok(url)
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<GoogleExchange, AuthError> {
        let response = self
            .http
            .post(self.token_endpoint.clone())
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|error| AuthError::Storage(format!("exchange google auth code: {error}")))?;
        let response = response
            .error_for_status()
            .map_err(|error| AuthError::Storage(format!("google token endpoint error: {error}")))?;
        let payload: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|error| AuthError::Storage(format!("decode google token response: {error}")))?;
        let subject = parse_subject_from_id_token(&payload.id_token)?;
        Ok(GoogleExchange {
            subject,
            access_token: payload.access_token,
            refresh_token: payload.refresh_token,
            expires_in: payload.expires_in,
            id_token: payload.id_token,
        })
    }

    pub async fn refresh(
        &self,
        refresh_token: &str,
    ) -> Result<GoogleExchange, AuthError> {
        let response = self
            .http
            .post(self.token_endpoint.clone())
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(|error| AuthError::Storage(format!("refresh google token: {error}")))?;
        let response = response
            .error_for_status()
            .map_err(|error| AuthError::Storage(format!("google refresh endpoint error: {error}")))?;
        let payload: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|error| AuthError::Storage(format!("decode google refresh response: {error}")))?;
        let subject = parse_subject_from_id_token(&payload.id_token)?;
        Ok(GoogleExchange {
            subject,
            access_token: payload.access_token,
            refresh_token: payload.refresh_token,
            expires_in: payload.expires_in,
            id_token: payload.id_token,
        })
    }
}

fn parse_subject_from_id_token(id_token: &str) -> Result<String, AuthError> {
    let mut parts = id_token.split('.');
    let _header = parts.next().ok_or_else(|| {
        AuthError::Storage("google id_token is missing a JWT header".to_string())
    })?;
    let payload = parts.next().ok_or_else(|| {
        AuthError::Storage("google id_token is missing a JWT payload".to_string())
    })?;
    let decoded = URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .map_err(|error| AuthError::Storage(format!("decode google id_token payload: {error}")))?;
    let claims: GoogleIdTokenClaims = serde_json::from_slice(&decoded)
        .map_err(|error| AuthError::Storage(format!("parse google id_token claims: {error}")))?;
    Ok(claims.sub)
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use serde_json::json;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{AuthorizeUrlRequest, GoogleProvider};

    #[test]
    fn google_authorize_url_includes_offline_access_and_pkce() {
        let provider = test_google_provider();
        let url = provider.authorize_url(&sample_request()).unwrap();
        assert!(url.as_str().contains("access_type=offline"));
        assert!(url.as_str().contains("code_challenge="));
    }

    #[tokio::test]
    async fn google_exchange_parses_subject_and_refresh_token() {
        let provider = mocked_google_provider().await;
        let token = provider.exchange_code("code", "verifier").await.unwrap();
        assert_eq!(token.subject, "google-subject-123");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-token"));
    }

    fn test_google_provider() -> GoogleProvider {
        GoogleProvider::new(
            "client-id".to_string(),
            "client-secret".to_string(),
            Url::parse("https://lab.example.com/auth/google/callback").unwrap(),
        )
        .unwrap()
    }

    async fn mocked_google_provider() -> GoogleProvider {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({
                    "access_token": "google-access-token",
                    "refresh_token": "refresh-token",
                    "expires_in": 3600,
                    "id_token": test_id_token(),
                })),
            )
            .mount(&server)
            .await;

        test_google_provider().with_endpoints(
            server.uri().parse::<Url>().unwrap(),
            server.uri().parse::<Url>().unwrap().join("/token").unwrap(),
        )
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
}
