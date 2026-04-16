use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::{Json, response::Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::error::AuthError;
use crate::google::AuthorizeUrlRequest;
use crate::session::{append_set_cookie, build_browser_session_cookie, create_browser_session};
use crate::state::AuthState;
use crate::types::{
    AuthorizationCodeRow, AuthorizationRequestRow, AuthorizeQuery, BrowserLoginQuery,
    BrowserLoginStateRow, CallbackQuery, ClientRegistrationRequest, ClientRegistrationResponse,
    RegisteredClient,
};
use crate::util::{fingerprint, now_unix, random_token};

const AUTH_REQUEST_TTL_SECS: i64 = 300;
const LAB_SCOPE: &str = "lab";

pub async fn browser_login(
    State(state): State<AuthState>,
    Query(query): Query<BrowserLoginQuery>,
) -> Result<Response, AuthError> {
    let return_to = sanitize_return_to(&state, query.return_to.as_deref());
    let provider_code_verifier = random_token(32)?;
    let provider_code_challenge =
        URL_SAFE_NO_PAD.encode(Sha256::digest(provider_code_verifier.as_bytes()));
    let request_state = random_token(24)?;
    let oauth_state_id = fingerprint(&request_state);

    state
        .store
        .insert_browser_login_state(BrowserLoginStateRow {
            state: request_state.clone(),
            return_to: return_to.clone(),
            provider_code_verifier,
            created_at: now_unix(),
            expires_at: now_unix() + AUTH_REQUEST_TTL_SECS,
        })
        .await?;

    let location = state.google.authorize_url(AuthorizeUrlRequest {
        state: request_state,
        scope: LAB_SCOPE.to_string(),
        code_challenge: provider_code_challenge,
        code_challenge_method: "S256".to_string(),
    })?;
    info!(
        oauth_state_id = %oauth_state_id,
        return_to = %return_to,
        "browser login redirected to upstream provider"
    );

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, location.to_string())],
    )
        .into_response())
}

pub async fn register_client(
    State(state): State<AuthState>,
    Json(request): Json<ClientRegistrationRequest>,
) -> Result<Json<ClientRegistrationResponse>, AuthError> {
    if request.redirect_uris.is_empty() {
        warn!("oauth register rejected: no redirect URIs provided");
        return Err(AuthError::Validation(
            "at least one redirect URI is required".to_string(),
        ));
    }
    for redirect_uri in &request.redirect_uris {
        if !is_allowed_redirect_uri(redirect_uri, &state.config.allowed_client_redirect_uris) {
            warn!(
                redirect_uri = %redirect_uri,
                allowed_patterns = ?state.config.allowed_client_redirect_uris,
                "oauth register rejected: redirect URI is not in the allowlist or loopback set"
            );
            return Err(AuthError::Validation(format!(
                "redirect URI `{redirect_uri}` must target a loopback host or match an allowed redirect pattern"
            )));
        }
    }

    let client = RegisteredClient {
        client_id: random_token(18)?,
        redirect_uris: request.redirect_uris,
        created_at: now_unix(),
    };
    state.store.register_client(client.clone()).await?;
    info!(
        client_id = %client.client_id,
        redirect_uri_count = client.redirect_uris.len(),
        redirect_uris = ?client.redirect_uris,
        "oauth client registration accepted"
    );
    Ok(Json(ClientRegistrationResponse {
        client_id: client.client_id,
        redirect_uris: client.redirect_uris,
        token_endpoint_auth_method: "none".to_string(),
    }))
}

pub async fn authorize(
    State(state): State<AuthState>,
    Query(query): Query<AuthorizeQuery>,
) -> Result<Response, AuthError> {
    validate_response_type(&query.response_type)?;
    let scope = validate_scope(&query.scope)?;
    let client_state_id = fingerprint(&query.state);
    info!(
        client_id = %query.client_id,
        redirect_uri = %query.redirect_uri,
        client_state_id = %client_state_id,
        requested_scope = %query.scope,
        "oauth authorize request received"
    );
    let client = state
        .store
        .find_client(&query.client_id)
        .await?
        .ok_or_else(|| {
            warn!(
                client_id = %query.client_id,
                client_state_id = %client_state_id,
                "oauth authorize rejected: unknown client_id"
            );
            AuthError::InvalidGrant("unknown client_id".to_string())
        })?;
    if !client
        .redirect_uris
        .iter()
        .any(|uri| uri == &query.redirect_uri)
    {
        warn!(
            client_id = %query.client_id,
            redirect_uri = %query.redirect_uri,
            client_state_id = %client_state_id,
            "oauth authorize rejected: redirect URI does not match registered client"
        );
        return Err(AuthError::Validation(
            "redirect_uri does not match the registered client".to_string(),
        ));
    }
    if query.code_challenge_method != "S256" {
        warn!(
            client_id = %query.client_id,
            client_state_id = %client_state_id,
            code_challenge_method = %query.code_challenge_method,
            "oauth authorize rejected: unsupported PKCE method"
        );
        return Err(AuthError::Validation(
            "code_challenge_method must be S256".to_string(),
        ));
    }

    let provider_code_verifier = random_token(32)?;
    let provider_code_challenge =
        URL_SAFE_NO_PAD.encode(Sha256::digest(provider_code_verifier.as_bytes()));
    let request_state = random_token(24)?;
    let oauth_state_id = fingerprint(&request_state);

    state
        .store
        .insert_authorization_request(AuthorizationRequestRow {
            state: request_state.clone(),
            client_id: query.client_id.clone(),
            redirect_uri: query.redirect_uri.clone(),
            client_state: query.state.clone(),
            scope: scope.clone(),
            provider_code_verifier,
            code_challenge: query.code_challenge.clone(),
            code_challenge_method: query.code_challenge_method.clone(),
            created_at: now_unix(),
            expires_at: now_unix() + AUTH_REQUEST_TTL_SECS,
        })
        .await?;

    let location = state.google.authorize_url(AuthorizeUrlRequest {
        state: request_state,
        scope,
        code_challenge: provider_code_challenge,
        code_challenge_method: "S256".to_string(),
    })?;
    info!(
        client_id = %query.client_id,
        redirect_uri = %query.redirect_uri,
        client_state_id = %client_state_id,
        oauth_state_id = %oauth_state_id,
        provider = "google",
        "oauth authorize request redirected to upstream provider"
    );
    debug!(
        client_id = %query.client_id,
        oauth_state_id = %oauth_state_id,
        location = %location,
        "oauth authorize redirect URL generated"
    );

    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, location.to_string())],
    )
        .into_response())
}

pub async fn callback(
    State(state): State<AuthState>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, AuthError> {
    let oauth_state_id = fingerprint(&query.state);
    info!(
        oauth_state_id = %oauth_state_id,
        provider = "google",
        "oauth callback received"
    );
    if let Some(login) = state.store.take_browser_login_state(&query.state).await? {
        let google = state
            .google
            .exchange_code(&query.code, &login.provider_code_verifier)
            .await?;
        let session = create_browser_session(&state, google.subject, google.email).await?;
        let mut response = Redirect::to(&login.return_to).into_response();
        append_set_cookie(
            &mut response,
            &build_browser_session_cookie(&state, &session.session_id),
        );
        info!(
            oauth_state_id = %oauth_state_id,
            return_to = %login.return_to,
            subject_id = %fingerprint(&session.subject),
            "browser login callback issued session cookie"
        );
        return Ok(response);
    }

    let request = state
        .store
        .take_authorization_request(&query.state)
        .await
        .map_err(|_| {
            warn!(
                oauth_state_id = %oauth_state_id,
                "oauth callback rejected: authorization state is invalid or expired"
            );
            AuthError::InvalidGrant("authorization state is invalid or expired".to_string())
        })?;
    info!(
        client_id = %request.client_id,
        redirect_uri = %request.redirect_uri,
        oauth_state_id = %oauth_state_id,
        client_state_id = %fingerprint(&request.client_state),
        "oauth callback state redeemed"
    );
    let google = state
        .google
        .exchange_code(&query.code, &request.provider_code_verifier)
        .await?;
    let subject_id = fingerprint(&google.subject);
    info!(
        client_id = %request.client_id,
        oauth_state_id = %oauth_state_id,
        subject_id = %subject_id,
        has_provider_refresh_token = google.refresh_token.is_some(),
        "oauth callback exchanged upstream code successfully"
    );
    let auth_code = random_token(24)?;
    let auth_code_id = fingerprint(&auth_code);
    state
        .store
        .insert_auth_code(AuthorizationCodeRow {
            code: auth_code.clone(),
            client_id: request.client_id,
            subject: google.subject,
            redirect_uri: request.redirect_uri.clone(),
            scope: request.scope,
            code_challenge: request.code_challenge,
            code_challenge_method: request.code_challenge_method,
            provider_refresh_token: google.refresh_token,
            created_at: now_unix(),
            expires_at: now_unix() + state.config.auth_code_ttl.as_secs() as i64,
        })
        .await?;
    info!(
        auth_code_id = %auth_code_id,
        oauth_state_id = %oauth_state_id,
        redirect_uri = %request.redirect_uri,
        "oauth callback issued local authorization code"
    );

    let redirect_uri = reqwest::Url::parse(&request.redirect_uri).map_err(|error| {
        AuthError::Storage(format!(
            "registered redirect_uri is not a valid URL: {error}"
        ))
    })?;
    let mut redirect_uri = redirect_uri;
    redirect_uri
        .query_pairs_mut()
        .append_pair("code", &auth_code)
        .append_pair("state", &request.client_state);
    debug!(
        auth_code_id = %auth_code_id,
        redirect_uri = %redirect_uri,
        "oauth callback redirecting client back to registered callback"
    );

    Ok(Redirect::to(redirect_uri.as_str()).into_response())
}

fn sanitize_return_to(state: &AuthState, requested: Option<&str>) -> String {
    let Some(requested) = requested.map(str::trim).filter(|value| !value.is_empty()) else {
        return "/".to_string();
    };
    if requested.starts_with('/') && !requested.starts_with("//") {
        return requested.to_string();
    }
    let Some(public_url) = state.config.public_url.as_ref() else {
        return "/".to_string();
    };
    let Ok(url) = reqwest::Url::parse(requested) else {
        return "/".to_string();
    };
    if url.scheme() != public_url.scheme()
        || url.host_str() != public_url.host_str()
        || url.port_or_known_default() != public_url.port_or_known_default()
    {
        return "/".to_string();
    }
    let mut normalized = url.path().to_string();
    if let Some(query) = url.query() {
        normalized.push('?');
        normalized.push_str(query);
    }
    if let Some(fragment) = url.fragment() {
        normalized.push('#');
        normalized.push_str(fragment);
    }
    normalized
}

fn validate_response_type(response_type: &str) -> Result<(), AuthError> {
    if response_type == "code" {
        Ok(())
    } else {
        warn!(
            response_type = %response_type,
            "oauth authorize rejected: unsupported response_type"
        );
        Err(AuthError::Validation(
            "response_type must be `code`".to_string(),
        ))
    }
}

fn validate_scope(scope: &str) -> Result<String, AuthError> {
    let normalized = scope.trim();
    if normalized.is_empty() {
        return Ok(LAB_SCOPE.to_string());
    }
    if normalized == LAB_SCOPE {
        return Ok(normalized.to_string());
    }
    warn!(scope = %normalized, "oauth authorize rejected: unsupported scope");
    Err(AuthError::Validation(format!(
        "scope must be `{LAB_SCOPE}`"
    )))
}

fn is_loopback_redirect(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    if url.scheme() != "http" {
        return false;
    }
    matches!(
        url.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("::1") | Some("[::1]")
    )
}

fn is_allowed_redirect_uri(value: &str, patterns: &[String]) -> bool {
    is_loopback_redirect(value)
        || patterns
            .iter()
            .any(|pattern| wildcard_matches(pattern, value))
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == value;
    }

    let anchored_start = !pattern.starts_with('*');
    let anchored_end = !pattern.ends_with('*');
    let non_empty_parts: Vec<&str> = parts.into_iter().filter(|part| !part.is_empty()).collect();
    if non_empty_parts.is_empty() {
        return true;
    }

    let mut cursor = 0usize;
    for (index, part) in non_empty_parts.iter().enumerate() {
        if index == 0 && anchored_start {
            if !value[cursor..].starts_with(part) {
                return false;
            }
            cursor += part.len();
            continue;
        }

        match value[cursor..].find(part) {
            Some(found) => cursor += found + part.len(),
            None => return false,
        }
    }

    if anchored_end && let Some(last) = non_empty_parts.last() {
        return value.ends_with(last);
    }

    true
}

#[cfg(test)]
pub mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use base64::Engine;
    use serde_json::json;
    use tower::util::ServiceExt;
    use url::Url;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::wildcard_matches;
    use crate::config::{AuthConfig, AuthMode, GoogleConfig};
    use crate::google::GoogleProvider;
    use crate::routes::router;
    use crate::state::AuthState;
    use crate::types::{AuthorizationRequestRow, RegisteredClient};

    use crate::util::now_unix;

    #[tokio::test]
    async fn register_accepts_public_dcr_and_enforces_loopback_redirects() {
        let app = router(test_auth_state().await);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "redirect_uris": ["http://127.0.0.1:7777/callback"]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let rejected = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "redirect_uris": ["https://claude.ai/api/mcp/auth_callback"]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn register_accepts_allowed_non_loopback_redirect_patterns() {
        let mut config = test_auth_config();
        config.allowed_client_redirect_uris =
            vec!["https://callback.tootie.tv/callback/*".to_string()];
        let app = router(test_auth_state_with_config(config).await);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({
                            "redirect_uris": ["https://callback.tootie.tv/callback/dookie"]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn wildcard_redirect_patterns_support_leading_and_infix_matches() {
        assert!(wildcard_matches(
            "https://callback.tootie.tv/callback/*",
            "https://callback.tootie.tv/callback/dookie"
        ));
        assert!(wildcard_matches(
            "*example.com/callback",
            "https://callback.example.com/callback"
        ));
        assert!(wildcard_matches(
            "https://callback.*.tv/callback/*",
            "https://callback.tootie.tv/callback/dookie"
        ));
        assert!(!wildcard_matches(
            "*example.com/callback",
            "https://callback.example.com/callback/extra"
        ));
    }

    #[tokio::test]
    async fn authorize_persists_full_state_and_redirects_to_google() {
        let app = router(test_auth_state_with_registered_client().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/authorize?response_type=code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&state=abc&scope=lab&code_challenge=pkce&code_challenge_method=S256")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FOUND);
        let location = response
            .headers()
            .get(header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.contains("accounts.google.com"));
    }

    #[tokio::test]
    async fn browser_login_starts_upstream_flow_and_persists_return_to_state() {
        let state = test_auth_state().await;
        let app = router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/login?return_to=%2Fgateways%2F%3Ftab%3Dlab")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FOUND);
        let location = Url::parse(
            response
                .headers()
                .get(header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let upstream_state = location
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .unwrap();
        let stored = state
            .store
            .take_browser_login_state(&upstream_state)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.return_to, "/gateways/?tab=lab");
    }

    #[tokio::test]
    async fn callback_rejects_expired_or_mismatched_state() {
        let app = router(test_auth_state_with_mock_google().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/google/callback?state=bad-state&code=upstream-code")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn browser_login_callback_sets_session_cookie_and_redirects_home() {
        let state = test_auth_state_with_mock_google().await;
        let app = router(state.clone());
        let login = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/login?return_to=%2Fgateways%2F")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let location = Url::parse(
            login
                .headers()
                .get(header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let upstream_state = location
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .unwrap();

        let callback = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/auth/google/callback?state={upstream_state}&code=upstream-code"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(callback.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            callback.headers().get(header::LOCATION).unwrap(),
            "/gateways/"
        );
        let cookie = callback
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .find_map(|value| value.to_str().ok())
            .unwrap();
        assert!(cookie.contains("lab_session="));
    }

    #[tokio::test]
    async fn authorize_rejects_missing_or_invalid_response_type() {
        let app = router(test_auth_state_with_registered_client().await);
        for uri in [
            "/authorize?client_id=client&redirect_uri=http://127.0.0.1:7777/callback&state=abc&scope=lab&code_challenge=pkce&code_challenge_method=S256",
            "/authorize?response_type=token&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&state=abc&scope=lab&code_challenge=pkce&code_challenge_method=S256",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    #[tokio::test]
    async fn authorize_rejects_invalid_scope() {
        let app = router(test_auth_state_with_registered_client().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/authorize?response_type=code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&state=abc&scope=lab:admin&code_challenge=pkce&code_challenge_method=S256")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn callback_rejects_expired_state() {
        let state = test_auth_state_with_registered_client().await;
        state
            .store
            .insert_authorization_request(AuthorizationRequestRow {
                state: "expired-state".to_string(),
                client_id: "client".to_string(),
                redirect_uri: "http://127.0.0.1:7777/callback".to_string(),
                client_state: "client-state".to_string(),
                scope: "lab".to_string(),
                provider_code_verifier: "provider-verifier".to_string(),
                code_challenge: "challenge".to_string(),
                code_challenge_method: "S256".to_string(),
                created_at: now_unix() - 300,
                expires_at: now_unix() - 1,
            })
            .await
            .unwrap();
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/auth/google/callback?state=expired-state&code=upstream-code")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    pub async fn test_auth_state() -> AuthState {
        test_auth_state_with_config(test_auth_config()).await
    }

    pub async fn test_auth_state_with_config(config: AuthConfig) -> AuthState {
        AuthState::new(config).await.unwrap()
    }

    fn test_auth_config() -> AuthConfig {
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        AuthConfig {
            mode: AuthMode::OAuth,
            public_url: Some(Url::parse("https://lab.example.com").unwrap()),
            sqlite_path: dir.path().join("auth.db"),
            key_path: dir.path().join("auth-jwt.pem"),
            bootstrap_secret: Some("bootstrap-secret".to_string()),
            allowed_client_redirect_uris: Vec::new(),
            google: GoogleConfig {
                client_id: "client-id".to_string(),
                client_secret: "client-secret".to_string(),
                callback_path: "/auth/google/callback".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
            ..AuthConfig::default()
        }
    }

    pub async fn test_auth_state_with_registered_client() -> AuthState {
        let state = test_auth_state().await;
        state
            .store
            .register_client(RegisteredClient {
                client_id: "client".to_string(),
                redirect_uris: vec!["http://127.0.0.1:7777/callback".to_string()],
                created_at: now_unix(),
            })
            .await
            .unwrap();
        state
    }

    async fn test_auth_state_with_mock_google() -> AuthState {
        let state = test_auth_state_with_registered_client().await;
        let server = Box::leak(Box::new(MockServer::start().await));
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "google-access-token",
                "refresh_token": "refresh-token",
                "expires_in": 3600,
                "id_token": test_id_token(),
            })))
            .mount(server)
            .await;
        state
            .store
            .insert_authorization_request(AuthorizationRequestRow {
                state: "good-state".to_string(),
                client_id: "client".to_string(),
                redirect_uri: "http://127.0.0.1:7777/callback".to_string(),
                client_state: "client-state".to_string(),
                scope: "lab".to_string(),
                provider_code_verifier: "provider-verifier".to_string(),
                code_challenge: "challenge".to_string(),
                code_challenge_method: "S256".to_string(),
                created_at: now_unix(),
                expires_at: now_unix() + 300,
            })
            .await
            .unwrap();
        let google = GoogleProvider::new(
            "client-id".to_string(),
            "client-secret".to_string(),
            Url::parse("https://lab.example.com/auth/google/callback").unwrap(),
        )
        .with_endpoints(
            server.uri().parse::<Url>().unwrap(),
            server.uri().parse::<Url>().unwrap().join("/token").unwrap(),
        );
        AuthState::for_tests(
            (*state.config).clone(),
            state.store.clone(),
            (*state.signing_keys).clone(),
            google,
        )
    }

    fn test_id_token() -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"alg":"none","typ":"JWT"}"#);
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"sub":"google-subject-123"}"#);
        format!("{header}.{payload}.")
    }
}
