use axum::{Json, extract::State};

use crate::state::AuthState;
use crate::types::{AuthorizationServerMetadata, ProtectedResourceMetadata};

pub async fn authorization_server_metadata(
    State(state): State<AuthState>,
) -> Json<AuthorizationServerMetadata> {
    let base = public_base_url(&state);
    Json(AuthorizationServerMetadata {
        issuer: base.clone(),
        authorization_endpoint: format!("{base}/authorize"),
        token_endpoint: format!("{base}/token"),
        registration_endpoint: format!("{base}/register"),
        jwks_uri: format!("{base}/jwks"),
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
        code_challenge_methods_supported: vec!["S256".to_string()],
        token_endpoint_auth_methods_supported: vec!["none".to_string()],
    })
}

pub async fn protected_resource_metadata(
    State(state): State<AuthState>,
) -> Json<ProtectedResourceMetadata> {
    let base = public_base_url(&state);
    Json(ProtectedResourceMetadata {
        resource: canonical_resource_url(&state),
        authorization_servers: vec![base],
        scopes_supported: vec!["lab".to_string()],
        bearer_methods_supported: vec!["header".to_string()],
    })
}

pub async fn jwks(State(state): State<AuthState>) -> Json<crate::jwt::JwksDocument> {
    Json(state.signing_keys.jwks().clone())
}

pub(crate) fn public_base_url(state: &AuthState) -> String {
    state
        .config
        .public_url
        .as_ref()
        .expect("oauth state must have public_url")
        .as_str()
        .trim_end_matches('/')
        .to_string()
}

pub fn canonical_resource_url(state: &AuthState) -> String {
    format!("{}/mcp", public_base_url(state))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    use crate::routes::router;

    use super::super::authorize::tests::test_auth_state;

    #[tokio::test]
    async fn authorization_server_metadata_exposes_lab_endpoints() {
        let app = router(test_auth_state().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/oauth-authorization-server")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["token_endpoint"], "https://lab.example.com/token");
    }

    #[tokio::test]
    async fn protected_resource_metadata_uses_canonical_mcp_resource_uri() {
        let app = router(test_auth_state().await);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/oauth-protected-resource")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["resource"], "https://lab.example.com/mcp");
    }
}
