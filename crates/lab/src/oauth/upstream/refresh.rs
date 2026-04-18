//! Refresh-on-401 coordination for upstream MCP OAuth clients.
//!
//! # Task 0 Spike Findings (2026-04-17)
//!
//! **rmcp version verified:** `1.4.0` (workspace `Cargo.toml` line ~53), built
//! with the `auth` and `transport-streamable-http-client-reqwest` features.
//!
//! **Integration path confirmed:** **Plan A — `AuthClient<reqwest::Client>`
//! auto-injects the bearer token.**
//!
//! Confirmed by inspecting
//! `rmcp-1.4.0/src/transport/common/auth/streamable_http_client.rs`:
//!
//! ```text
//! impl<C> StreamableHttpClient for AuthClient<C>
//! where C: StreamableHttpClient + Send + Sync,
//! {
//!     async fn post_message(&self, …, mut auth_token: Option<String>, …) {
//!         if auth_token.is_none() {
//!             auth_token = Some(self.get_access_token().await?);
//!         }
//!         self.http_client.post_message(uri, message, session_id,
//!             auth_token, custom_headers).await
//!     }
//!     // delete_session and get_stream follow the same pattern
//! }
//! ```
//!
//! Because `reqwest::Client` itself implements `StreamableHttpClient` (in
//! `rmcp-1.4.0/src/transport/common/reqwest/streamable_http_client.rs`),
//! `AuthClient<reqwest::Client>` is a drop-in `StreamableHttpClient` we can
//! pass to `StreamableHttpClientTransport::with_client(...)`. Downstream code
//! sends `auth_token: None`; `AuthClient` fills it in via
//! `AuthorizationManager::get_access_token()`.
//!
//! ## Refresh semantics (read carefully — this drives Task 2)
//!
//! `AuthorizationManager::get_access_token()` (auth.rs L1238) ONLY refreshes
//! when the LOCAL CLOCK says the token has less than `REFRESH_BUFFER_SECS`
//! (30 s) remaining. It does **not** react to a 401 from the resource server.
//!
//! Concretely, if the upstream MCP server responds 401 while our locally
//! cached token still looks valid (or has no `expires_in` at all):
//!   - `AuthClient::post_message` returns the 401 wrapped in
//!     `StreamableHttpError::AuthRequired` (if the server emitted a
//!     `WWW-Authenticate: Bearer` header) or some other transport error.
//!   - **rmcp does not automatically call `auth_manager.refresh_token()`.**
//!   - **rmcp does not automatically retry the request.**
//!
//! `AuthorizationManager::refresh_token()` is therefore **manual**. Task 2
//! must supply the refresh-on-401 loop around `AuthClient`.
//!
//! ## Implications for Task 2
//!
//! Task 2 must implement (sketch, do not treat as final):
//! 1. A `RefreshCoordinator` that wraps `AuthClient<reqwest::Client>` and,
//!    on `StreamableHttpError::AuthRequired` / HTTP 401, calls
//!    `auth_manager.refresh_token()` once, then retries the call exactly
//!    once. Second 401 → propagate.
//! 2. If `refresh_token()` fails with `AuthError::AuthorizationRequired`
//!    (no refresh token, refresh token revoked, etc.), surface the
//!    reauthorization prompt to the operator — a stored credential is now
//!    useless.
//! 3. Per-(upstream, subject) serialization: hold an `AuthorizationManager`
//!    behind a `Mutex` or single-flight gate so concurrent MCP calls from
//!    the same subject don't trigger parallel refreshes against the AS.
//!
//! ## Why NOT Plan B
//!
//! Plan B (a custom `StreamableHttpClient` wrapper that calls
//! `auth_manager.get_access_token()` before each request and passes
//! `auth_header: Some(format!("Bearer {token}"))`) is strictly inferior:
//! `AuthClient` already does exactly this, and is kept in sync with upstream
//! rmcp changes. The spike (`crates/lab/examples/spike_rmcp_auth_client.rs`)
//! drives a wiremock upstream, asserts that the bearer header is injected
//! without any explicit `auth_header` arg, and asserts that a 401 is NOT
//! followed by an automatic refresh.
//!
//! # Task 2 hand-off
//!
//! When Task 2 picks this up:
//! - Wire `pub mod upstream;` into `crates/lab/src/oauth.rs`.
//! - Replace this stub with the real `RefreshCoordinator` implementation.
//! - Keep the header docblock above (trim the Task 0 framing, keep the
//!   rmcp refresh-semantics facts) so future readers don't have to relearn
//!   the 401 behavior.
