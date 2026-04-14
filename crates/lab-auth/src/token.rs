use axum::extract::{Form, State};
use axum::{Json, response::IntoResponse};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::error::AuthError;
use crate::jwt::AccessClaims;
use crate::state::AuthState;
use crate::types::{RefreshTokenRow, TokenRequest, TokenResponse};
use crate::util::{now_unix, random_token};

pub async fn token(
    State(state): State<AuthState>,
    Form(request): Form<TokenRequest>,
) -> Result<impl IntoResponse, AuthError> {
    match request.grant_type.as_str() {
        "authorization_code" => authorization_code_grant(state, request).await.map(Json),
        "refresh_token" => refresh_token_grant(state, request).await.map(Json),
        other => Err(AuthError::Validation(format!(
            "unsupported grant_type `{other}`"
        ))),
    }
}

async fn authorization_code_grant(
    state: AuthState,
    request: TokenRequest,
) -> Result<TokenResponse, AuthError> {
    let code = require_field(request.code, "code")?;
    let client_id = require_field(request.client_id, "client_id")?;
    let redirect_uri = require_field(request.redirect_uri, "redirect_uri")?;
    let code_verifier = require_field(request.code_verifier, "code_verifier")?;

    let row = state.store.redeem_auth_code(&code).await?;
    if row.client_id != client_id {
        return Err(AuthError::InvalidGrant(
            "client_id does not match the authorization code".to_string(),
        ));
    }
    if row.redirect_uri != redirect_uri {
        return Err(AuthError::InvalidGrant(
            "redirect_uri does not match the authorization code".to_string(),
        ));
    }
    if row.code_challenge_method != "S256" {
        return Err(AuthError::InvalidGrant(
            "authorization code uses an unsupported PKCE method".to_string(),
        ));
    }
    if pkce_challenge(&code_verifier) != row.code_challenge {
        return Err(AuthError::InvalidGrant(
            "code_verifier does not match the authorization code".to_string(),
        ));
    }

    let refresh_token = random_token(24)?;
    state
        .store
        .upsert_refresh_token(RefreshTokenRow {
            refresh_token: refresh_token.clone(),
            client_id: row.client_id.clone(),
            subject: row.subject.clone(),
            scope: row.scope.clone(),
            provider_refresh_token: row.provider_refresh_token,
            created_at: now_unix(),
            expires_at: now_unix() + state.config.refresh_token_ttl.as_secs() as i64,
        })
        .await?;

    Ok(build_token_response(
        &state,
        row.client_id,
        row.subject,
        row.scope,
        refresh_token,
    )?)
}

async fn refresh_token_grant(
    state: AuthState,
    request: TokenRequest,
) -> Result<TokenResponse, AuthError> {
    let client_id = require_field(request.client_id, "client_id")?;
    let refresh_token = require_field(request.refresh_token, "refresh_token")?;
    let stored = state
        .store
        .find_refresh_token(&refresh_token)
        .await?
        .ok_or_else(|| AuthError::InvalidGrant("unknown refresh_token".to_string()))?;
    if stored.client_id != client_id {
        return Err(AuthError::InvalidGrant(
            "client_id does not match the refresh token".to_string(),
        ));
    }

    let google = if let Some(provider_refresh_token) = stored.provider_refresh_token.clone() {
        Some(state.google.refresh(&provider_refresh_token).await?)
    } else {
        None
    };

    state
        .store
        .upsert_refresh_token(RefreshTokenRow {
            refresh_token: refresh_token.clone(),
            client_id: stored.client_id.clone(),
            subject: google
                .as_ref()
                .map_or_else(|| stored.subject.clone(), |token| token.subject.clone()),
            scope: stored.scope.clone(),
            provider_refresh_token: google
                .and_then(|token| token.refresh_token)
                .or(stored.provider_refresh_token),
            created_at: stored.created_at,
            expires_at: now_unix() + state.config.refresh_token_ttl.as_secs() as i64,
        })
        .await?;

    Ok(build_token_response(
        &state,
        stored.client_id,
        stored.subject,
        stored.scope,
        refresh_token,
    )?)
}

fn build_token_response(
    state: &AuthState,
    client_id: String,
    subject: String,
    scope: String,
    refresh_token: String,
) -> Result<TokenResponse, AuthError> {
    let base = state
        .config
        .public_url
        .as_ref()
        .expect("oauth state must have public_url")
        .as_str()
        .trim_end_matches('/')
        .to_string();
    let now = now_unix() as usize;
    let access_token = state.signing_keys.issue_access_token(AccessClaims {
        iss: base.clone(),
        sub: subject,
        aud: base,
        exp: now + state.config.access_token_ttl.as_secs() as usize,
        iat: now,
        jti: random_token(18)?,
        scope: scope.clone(),
        azp: client_id,
    })?;
    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.access_token_ttl.as_secs(),
        refresh_token,
        scope,
    })
}

fn require_field(value: Option<String>, field: &str) -> Result<String, AuthError> {
    value.ok_or_else(|| AuthError::Validation(format!("missing `{field}` parameter")))
}

fn pkce_challenge(code_verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::util::ServiceExt;

    use crate::routes::router;

    use super::super::authorize::tests::test_auth_state_with_registered_client;

    #[tokio::test]
    async fn token_endpoint_mints_lab_jwt_and_refresh_token() {
        let state = test_auth_state_with_registered_client().await;
        seed_authorization_code(&state).await;
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from("grant_type=authorization_code&code=lab-code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&code_verifier=verifier"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["access_token"].is_string());
        assert!(json["refresh_token"].is_string());
    }

    #[tokio::test]
    async fn token_endpoint_redeems_authorization_code_once() {
        let state = test_auth_state_with_registered_client().await;
        seed_authorization_code(&state).await;
        let app = router(state);
        let (a, b) = tokio::join!(
            app.clone().oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from("grant_type=authorization_code&code=lab-code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&code_verifier=verifier"))
                    .unwrap()
            ),
            app.oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from("grant_type=authorization_code&code=lab-code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&code_verifier=verifier"))
                    .unwrap()
            )
        );
        let a = a.unwrap();
        let b = b.unwrap();
        assert!(a.status() == StatusCode::OK || b.status() == StatusCode::OK);
        assert!(a.status() == StatusCode::BAD_REQUEST || b.status() == StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn token_endpoint_rejects_expired_authorization_code() {
        let state = test_auth_state_with_registered_client().await;
        seed_authorization_code_with_expiry(&state, crate::util::now_unix() - 1).await;
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from("grant_type=authorization_code&code=lab-code&client_id=client&redirect_uri=http://127.0.0.1:7777/callback&code_verifier=verifier"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn token_endpoint_rejects_expired_refresh_token() {
        let state = test_auth_state_with_registered_client().await;
        state
            .store
            .upsert_refresh_token(crate::types::RefreshTokenRow {
                refresh_token: "refresh-token".to_string(),
                client_id: "client".to_string(),
                subject: "google-subject-123".to_string(),
                scope: "lab".to_string(),
                provider_refresh_token: Some("provider-refresh".to_string()),
                created_at: crate::util::now_unix() - 3600,
                expires_at: crate::util::now_unix() - 1,
            })
            .await
            .unwrap();
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from(
                        "grant_type=refresh_token&refresh_token=refresh-token&client_id=client",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn token_endpoint_rejects_refresh_token_client_mismatch() {
        let state = test_auth_state_with_registered_client().await;
        state
            .store
            .upsert_refresh_token(crate::types::RefreshTokenRow {
                refresh_token: "refresh-token".to_string(),
                client_id: "client".to_string(),
                subject: "google-subject-123".to_string(),
                scope: "lab".to_string(),
                provider_refresh_token: Some("provider-refresh".to_string()),
                created_at: crate::util::now_unix() - 60,
                expires_at: crate::util::now_unix() + 3600,
            })
            .await
            .unwrap();
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/token")
                    .header(
                        header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded",
                    )
                    .body(Body::from(
                        "grant_type=refresh_token&refresh_token=refresh-token&client_id=other-client",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn seed_authorization_code(state: &crate::state::AuthState) {
        seed_authorization_code_with_expiry(state, 4_102_444_800).await;
    }

    async fn seed_authorization_code_with_expiry(
        state: &crate::state::AuthState,
        expires_at: i64,
    ) {
        state
            .store
            .insert_auth_code(crate::types::AuthorizationCodeRow {
                code: "lab-code".to_string(),
                client_id: "client".to_string(),
                subject: "google-subject-123".to_string(),
                redirect_uri: "http://127.0.0.1:7777/callback".to_string(),
                scope: "lab".to_string(),
                code_challenge: super::pkce_challenge("verifier"),
                code_challenge_method: "S256".to_string(),
                provider_refresh_token: Some("provider-refresh".to_string()),
                created_at: 1_700_000_000,
                expires_at,
            })
            .await
            .unwrap();
    }
}
