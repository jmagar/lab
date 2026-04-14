use axum::Router;
use axum::routing::{get, post};

use crate::authorize::{authorize, callback, register_client};
use crate::metadata::{authorization_server_metadata, jwks, protected_resource_metadata};
use crate::state::AuthState;
use crate::token::token;

pub fn router(state: AuthState) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata),
        )
        .route("/jwks", get(jwks))
        .route("/register", post(register_client))
        .route("/authorize", get(authorize))
        .route("/auth/google/callback", get(callback))
        .route("/token", post(token))
        .with_state(state)
}
