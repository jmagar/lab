# HTTP Auth Modes

Lab supports two HTTP auth modes:

- `LAB_AUTH_MODE=bearer`
  Preserve the existing static bearer-token flow with `LAB_MCP_HTTP_TOKEN`.
- `LAB_AUTH_MODE=oauth`
  Run an internal Google-backed OAuth authorization server that issues `lab` JWT access tokens and exposes JWKS plus RFC 9728 metadata.

This document covers mode selection, startup behavior, registration and token flow, JWT validation, and operator-facing constraints.

## Configuration

OAuth mode is configured through env vars and/or `config.toml`. Env vars take precedence over config file values.

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LAB_AUTH_MODE` | no | `bearer` or `oauth`. Defaults to `bearer`. |
| `LAB_MCP_HTTP_TOKEN` | bearer mode | Static bearer token for protected HTTP routes. |
| `LAB_PUBLIC_URL` | oauth mode | Public base URL for metadata, callback construction, and JWT issuer/audience. |
| `LAB_GOOGLE_CLIENT_ID` | oauth mode | Google OAuth client ID. |
| `LAB_GOOGLE_CLIENT_SECRET` | oauth mode | Google OAuth client secret. |
| `LAB_AUTH_SQLITE_PATH` | no | Override path for the SQLite auth database. |
| `LAB_AUTH_KEY_PATH` | no | Override path for the persisted JWT signing key. |
| `LAB_AUTH_ALLOWED_REDIRECT_URIS` | no | Comma-separated redirect URI patterns allowed for dynamic client registration in addition to loopback callbacks. |
| `LAB_GOOGLE_CALLBACK_PATH` | no | Callback path appended to `LAB_PUBLIC_URL`. Defaults to `/auth/google/callback`. |
| `LAB_GOOGLE_SCOPES` | no | Comma-separated Google scopes. Defaults to `openid,email,profile`. |

## Startup Behavior

When OAuth mode is configured, `lab serve --transport http` performs these steps at startup:

1. Validate that `LAB_PUBLIC_URL` and Google credentials are present.
2. Open the SQLite auth store in WAL mode with a non-zero busy timeout.
3. Load or generate the persisted RSA signing key.
4. Build the concrete Google provider callback URL from `LAB_PUBLIC_URL` and `LAB_GOOGLE_CALLBACK_PATH`.

Startup fails closed if any of those steps fail.

Startup also fails if:

- `LAB_AUTH_MODE=oauth` is set without `LAB_PUBLIC_URL`
- Google client credentials are missing
- the auth database or signing key has insecure file permissions

## Registration and Authorize Flow

OAuth mode exposes:

- `POST /register`
- `GET /authorize`
- `GET /auth/google/callback`
- `POST /token`

Registration rules in the initial launch:

- loopback redirect URIs are always accepted
- optional non-loopback redirect URI patterns can be allowed with `LAB_AUTH_ALLOWED_REDIRECT_URIS` or `[auth].allowed_client_redirect_uris`
- unlisted public HTTPS redirect URIs are rejected

Flow summary:

1. A client registers a loopback redirect URI or one that matches the configured allowlist.
2. The client sends the user to `/authorize` with `response_type=code`.
3. `lab` stores the request state, generates PKCE data, and redirects to Google.
4. Google redirects back to `/auth/google/callback`.
5. `lab` exchanges the Google code server-side, stores a local authorization code, and redirects the client back to its registered redirect URI with the local code.
6. The client exchanges that local code at `/token` for a `lab` access token and, when Google granted offline access, a `lab` refresh token.

Google access and refresh tokens remain server-side only.

Google-specific notes:

- `lab` sends `access_type=offline` when redirecting to Google so the provider can issue a refresh token
- `lab` also sends `prompt=consent` so a fresh Google consent flow can return a new refresh token after the app was previously authorized without offline access
- if Google still does not return an upstream refresh token, `lab` omits `refresh_token` from its token response and later refresh grants fail closed

## Browser-Local Callback Forwarding

`lab` also ships a local OAuth callback forwarder for browser-side machines:

```bash
lab oauth relay-local --machine dookie --port 38935
lab oauth relay-local --forward-base http://100.88.16.79:38935/callback/dookie --port 38935
```

This helper exists for cases where:

- the browser receives a loopback redirect on one machine
- the actual OAuth client callback listener is running on another machine
- you need to forward the final callback request without reimplementing the OAuth flow

Important constraints:

- `relay-local` binds only to `127.0.0.1:<port>` on the browser machine
- it forwards only the final callback request
- it does not mint tokens, store PKCE state, or complete the OAuth exchange itself
- the real client listener must already be running and reachable before the callback arrives

## Device Runtime Relay Start

The same local relay can be started remotely on a fleet device through:

```http
POST /v1/device/oauth/relay/start
```

Example body:

```json
{
  "bind_addr": "127.0.0.1:38935",
  "target_url": "http://100.88.16.79:38935/callback/dookie",
  "default_port": 38935,
  "request_timeout_ms": 30000
}
```

This reuses the existing local relay implementation. It does not change OAuth token issuance or PKCE handling.

### Using non-loopback redirect URIs

Loopback redirect URIs are always accepted by `lab-auth`. Public or non-loopback redirect URIs are
rejected unless they match an allowlisted pattern.

Configure extra allowed redirect URI patterns with either:

- `LAB_AUTH_ALLOWED_REDIRECT_URIS`
- `[auth].allowed_client_redirect_uris`

Example:

```env
LAB_AUTH_ALLOWED_REDIRECT_URIS=https://callback.tootie.tv/callback/*
```

```toml
[auth]
allowed_client_redirect_uris = ["https://callback.tootie.tv/callback/*"]
```

Patterns support simple `*` wildcards. Use this only for redirect URIs you explicitly operate or
trust.

## Runtime JWT Validation

Every request to a protected route (`/v1/*`, `/mcp`) must include an `Authorization: Bearer <token>` header.

Validation steps:

1. Decode the JWT header to extract the `kid` (key ID).
2. Look up the signing key in the cached JWKS.
3. If the `kid` is unknown, trigger an eager JWKS refresh (see caching below).
4. Validate the JWT signature using one of the supported algorithms.
5. Validate the `iss` claim matches the configured issuer.
6. Validate the `aud` claim matches the configured audience.
7. Extract scopes from the `scope` claim (space-separated string) or the `scp` claim (JSON array).

### Supported Algorithm

- RS256

### Scopes

Current `lab` tokens use the standard space-delimited `scope` claim.

### AuthContext

On successful validation, an `AuthContext` is injected into the request extensions:

- `sub` — the authenticated user/client identifier from the `sub` claim.
- `scopes` — granted scopes.
- `issuer` — token issuer.

Downstream handlers can read `AuthContext` from request extensions for audit trails and scope-gated access.

## Token Exchange

`POST /token` supports:

- `grant_type=authorization_code`
- `grant_type=refresh_token`

Current constraints:

- authorization-code redemption is atomic and single-use
- `refresh_token` is only issued when Google returned an upstream refresh token
- refresh grants are rejected if the local token is not backed by an upstream refresh token
- refresh tokens do not rotate in this batch
- `/revoke` is not implemented in this batch

## RFC 9728 Protected Resource Metadata

Lab exposes a metadata endpoint so MCP clients can discover which authorization server to use:

```http
GET /.well-known/oauth-protected-resource
```

This endpoint is **unauthenticated** — clients need it before they have a token.

Response:

```json
{
  "resource": "https://lab.example.com",
  "authorization_servers": ["https://lab.example.com"],
  "scopes_supported": ["lab"],
  "bearer_methods_supported": ["header"]
}
```

### WWW-Authenticate Header

When a request fails authentication (401), the response includes:

```http
WWW-Authenticate: Bearer resource_metadata="https://lab.example.com/.well-known/oauth-protected-resource"
```

This header is only included when `LAB_PUBLIC_URL` is configured. If not, the header is omitted rather than advertising localhost.

## Auth Precedence

When both static bearer and OAuth are configured, auth is checked in this order:

1. **Static bearer token** — constant-time comparison via `LAB_MCP_HTTP_TOKEN`. If it matches, the request is authenticated with implicit `lab:read` and `lab:admin` scopes.
2. **OAuth JWT** — if the static bearer check fails (or no static token is configured), the token is validated as a JWT against the cached JWKS. OAuth-issued tokens currently carry the single supported scope `lab`.
3. **401** — if both checks fail (or neither auth method is configured for the token presented).

Static bearer tokens bypass all JWT validation. This allows operators to use a simple token for automation while also supporting OAuth for interactive or multi-tenant use.

For device-runtime background traffic, the supported auth path in this implementation is the shared static bearer token when `LAB_MCP_HTTP_TOKEN` is configured.

## Safety Gate

Lab refuses to bind on a non-localhost address without any auth configured:

```text
refusing to bind HTTP on 0.0.0.0:8765 without authentication.
Set LAB_MCP_HTTP_TOKEN or LAB_AUTH_MODE=oauth, or bind to 127.0.0.1 for local-only access.
```

Loopback hosts exempt from this check: `127.0.0.1`, `::1`, `[::1]`, `localhost`.

## Example: Deploying with OAuth

```bash
# In ~/.lab/.env
LAB_MCP_TRANSPORT=http
LAB_MCP_HTTP_HOST=0.0.0.0
LAB_MCP_HTTP_PORT=8765
LAB_AUTH_MODE=oauth
LAB_PUBLIC_URL=https://lab.example.com
LAB_GOOGLE_CLIENT_ID=google-client-id
LAB_GOOGLE_CLIENT_SECRET=google-client-secret

# Start
lab serve --transport http
```

Verify the metadata endpoint:

```bash
curl https://lab.example.com/.well-known/oauth-protected-resource
```

Call a protected endpoint with a `lab` access token:

```bash
curl -H "Authorization: Bearer eyJhbG..." \
     https://lab.example.com/v1/radarr \
     -d '{"action":"help"}'
```

## Related Docs

- [CONFIG.md](./CONFIG.md) — config loading and env var conventions
- [TRANSPORT.md](./TRANSPORT.md) — HTTP transport setup and middleware
- [ERRORS.md](./ERRORS.md) — `auth_failed` error kind
- [RMCP.md](./RMCP.md) — RMCP auth ownership contract
