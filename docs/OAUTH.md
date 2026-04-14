# OAuth 2.1 Resource Server

Lab acts as an OAuth 2.1 resource server (RFC 9728). It validates tokens issued by an external authorization server. It does not issue tokens.

This document covers configuration, startup behavior, runtime validation, JWKS caching, and failure modes.

## Configuration

OAuth is configured through env vars and/or `config.toml`. Env vars take precedence over config file values.

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LAB_OAUTH_ISSUER` | yes | OIDC issuer URL. Must use HTTPS. |
| `LAB_OAUTH_AUDIENCE` | yes | Expected `aud` claim (RFC 8707). |
| `LAB_OAUTH_CLIENT_ID` | no | Optional `azp` claim validation. |
| `LAB_RESOURCE_URL` | yes | Public URL of this lab instance. Required when OAuth is enabled. |

### config.toml

```toml
[oauth]
issuer = "https://auth.example.com"
audience = "https://lab.example.com"
client_id = "optional-client-id"
resource_url = "https://lab.example.com"
```

Env vars override every field in the `[oauth]` section. If neither `LAB_OAUTH_ISSUER` nor `[oauth].issuer` is set, OAuth is not configured and lab falls back to static bearer auth only.

## Startup Behavior

When OAuth is configured, `lab serve --transport http` performs these steps at startup:

1. Fetch `{issuer}/.well-known/openid-configuration` (OIDC discovery).
2. Extract `jwks_uri` from the discovery response.
3. Fetch the initial JWKS from `jwks_uri`.

Startup **fails** if any of these steps fail. Lab never degrades to unauthenticated operation when OAuth is configured. The error message includes the issuer URL and the specific failure.

Startup also **fails** if:

- `LAB_OAUTH_ISSUER` is set but empty.
- `LAB_OAUTH_AUDIENCE` is not set when `LAB_OAUTH_ISSUER` is present.
- `LAB_OAUTH_ISSUER` does not use HTTPS.
- `LAB_RESOURCE_URL` is not set when OAuth is configured.

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

### Supported Algorithms

- RS256, RS384, RS512
- ES256, ES384

### Scopes

Lab recognizes two scopes:

- `lab:read` — read-only access
- `lab:admin` — full access including destructive operations

Scopes are read from the JWT `scope` claim (space-delimited string) or the `scp` claim (JSON string array). If both are present, `scope` takes precedence.

### AuthContext

On successful validation, an `AuthContext` is injected into the request extensions:

- `sub` — the authenticated user/client identifier from the `sub` claim.
- `scopes` — granted scopes.
- `issuer` — token issuer.

Downstream handlers can read `AuthContext` from request extensions for audit trails and scope-gated access.

## JWKS Caching

The JWKS cache uses a stale-while-revalidate strategy.

### TTL-Based Background Refresh

- Cache TTL: 1 hour.
- When `validate_jwt()` is called and the cache TTL has expired, a background refresh is attempted.
- If the refresh fails, stale keys continue to be served. An error is returned only if the cache is completely empty.

### Eager Refresh on Unknown `kid`

- When a JWT presents a `kid` not in the cache, `ensure_kid()` triggers an immediate refresh.
- A semaphore (capacity 1) prevents thundering herd — only one refresh runs at a time.
- Double-checked locking: after acquiring the semaphore, the cache is re-checked before fetching.
- If the eager refresh fails and stale keys exist, they are served with a warning logged.
- If the cache is empty (no keys at all), the error propagates.

### Failure Modes

| Scenario | Behavior |
|----------|----------|
| OIDC discovery fails at startup | Startup aborted with error |
| Initial JWKS fetch fails at startup | Startup aborted with error |
| Background refresh fails (stale keys exist) | Stale keys served, warning logged |
| Background refresh fails (no cached keys) | Error returned to caller |
| Eager refresh fails (stale keys exist) | Stale keys served, warning logged |
| Eager refresh fails (no cached keys) | `auth_failed` error |
| JWT `kid` not found after refresh | `auth_failed` error |

## RFC 9728 Protected Resource Metadata

Lab exposes a metadata endpoint so MCP clients can discover which authorization server to use:

```
GET /.well-known/oauth-protected-resource
```

This endpoint is **unauthenticated** — clients need it before they have a token.

Response:

```json
{
  "resource": "https://lab.example.com",
  "authorization_servers": ["https://auth.example.com"],
  "scopes_supported": ["lab:read", "lab:admin"],
  "bearer_methods_supported": ["header"]
}
```

### WWW-Authenticate Header

When a request fails authentication (401), the response includes:

```
WWW-Authenticate: Bearer resource_metadata="https://lab.example.com/.well-known/oauth-protected-resource"
```

This header is only included when `LAB_RESOURCE_URL` is configured. If not, the header is omitted rather than advertising localhost.

## Auth Precedence

When both static bearer and OAuth are configured, auth is checked in this order:

1. **Static bearer token** — constant-time comparison via `LAB_MCP_HTTP_TOKEN`. If it matches, the request is authenticated with implicit `lab:read` and `lab:admin` scopes.
2. **OAuth JWT** — if the static bearer check fails (or no static token is configured), the token is validated as a JWT against the cached JWKS.
3. **401** — if both checks fail (or neither auth method is configured for the token presented).

Static bearer tokens bypass all JWT validation. This allows operators to use a simple token for automation while also supporting OAuth for interactive or multi-tenant use.

## Safety Gate

Lab refuses to bind on a non-localhost address without any auth configured:

```
refusing to bind HTTP on 0.0.0.0:8765 without authentication.
Set LAB_MCP_HTTP_TOKEN or LAB_OAUTH_ISSUER, or bind to 127.0.0.1 for local-only access.
```

Loopback hosts exempt from this check: `127.0.0.1`, `::1`, `[::1]`, `localhost`.

## Example: Deploying with OAuth

```bash
# In ~/.lab/.env
LAB_MCP_TRANSPORT=http
LAB_MCP_HTTP_HOST=0.0.0.0
LAB_MCP_HTTP_PORT=8765
LAB_OAUTH_ISSUER=https://auth.example.com
LAB_OAUTH_AUDIENCE=https://lab.example.com
LAB_RESOURCE_URL=https://lab.example.com

# Start
lab serve --transport http
```

Verify the metadata endpoint:

```bash
curl https://lab.example.com/.well-known/oauth-protected-resource
```

Call a protected endpoint with a JWT:

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
