# Transport Configuration

Lab supports two MCP transports: stdio and streamable HTTP. Both expose the same server behavior — transport choice does not change the catalog, schemas, envelopes, or destructive-op policy.

## Stdio (Default)

Stdio is the default transport. Used by Claude Desktop, IDE extensions, and any MCP client that launches lab as a child process.

No authentication is required — security is provided by process-level isolation. The parent process owns the stdio pipes and controls access.

```bash
lab serve                      # stdio (default)
lab serve --transport stdio    # explicit
```

Or via env:

```bash
LAB_MCP_TRANSPORT=stdio lab serve
```

No network listener is opened. No host, port, or auth configuration is needed.

## Streamable HTTP

The HTTP transport mounts the MCP protocol at `/mcp` inside the axum HTTP server, alongside the REST API at `/v1/*`.

```bash
lab serve --transport http
```

### Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `LAB_MCP_TRANSPORT` | `stdio` | Transport selection. Set to `http` for network access. |
| `LAB_MCP_HTTP_HOST` | `127.0.0.1` | Bind address. |
| `LAB_MCP_HTTP_PORT` | `8765` | Bind port. |
| `LAB_MCP_HTTP_TOKEN` | — | Static bearer token for authentication. |
| `LAB_MCP_SESSION_TTL_SECS` | `300` | Session keep-alive TTL in seconds. |
| `LAB_MCP_STATEFUL` | `true` | Whether to use stateful MCP sessions. |
| `LAB_MCP_ALLOWED_HOSTS` | — | Comma-separated hostnames for DNS rebinding protection. |
| `LAB_PUBLIC_URL` | — | Public URL of this lab instance. Its host is added to the allowed-host list in OAuth mode. |
| `LAB_CORS_ORIGINS` | — | Comma-separated CORS origin allowlist. |

Config TOML equivalents (env vars take precedence):

```toml
[mcp]
transport = "http"
host = "127.0.0.1"
port = 8765
```

CLI flags take precedence over env vars, which take precedence over config.toml:

1. `--host`, `--port`, `--transport` (CLI)
2. `LAB_MCP_HTTP_HOST`, `LAB_MCP_HTTP_PORT`, `LAB_MCP_TRANSPORT` (env)
3. `mcp.host`, `mcp.port`, `mcp.transport` (config.toml)
4. Defaults: `127.0.0.1`, `8765`, `stdio`

### Session Management

The HTTP transport uses RMCP's `StreamableHttpService` with a `LocalSessionManager`.

- `LAB_MCP_SESSION_TTL_SECS` controls the session keep-alive duration (default: 300 seconds / 5 minutes).
- `LAB_MCP_STATEFUL` controls whether stateful sessions are used (default: `true`). Set to `false` for stateless operation.

### DNS Rebinding Protection

The HTTP transport validates the `Host` header against an allowed hosts list. This prevents DNS rebinding attacks where a malicious page redirects its hostname to `127.0.0.1`.

Allowed hosts are assembled from:

1. **Always included:** `localhost`, `127.0.0.1`, `::1`.
2. **`LAB_MCP_ALLOWED_HOSTS`** — comma-separated additional hostnames.
3. **`LAB_PUBLIC_URL`** — when OAuth mode is enabled, the hostname is automatically extracted and added.

Wildcard (`*`) is rejected with a warning — it would disable Host header validation entirely.

### Authentication

Protected routes (`/v1/*` and `/mcp`) require authentication when a static bearer token or OAuth mode is configured. Unauthenticated routes (`/health`, `/ready`, and OAuth metadata endpoints) are always accessible.

Auth methods (see [OAUTH.md](./OAUTH.md) for details):

- **Static bearer token** via `LAB_MCP_HTTP_TOKEN` — constant-time comparison.
- **OAuth mode** via `LAB_AUTH_MODE=oauth`, `LAB_PUBLIC_URL`, and Google client credentials.
- Both can be active simultaneously. Static bearer is checked first.
- If neither auth method is configured, the router permits local loopback requests only; non-loopback binds are rejected by the safety gate below.

### Safety Gate

Lab refuses to bind on a non-localhost address without auth:

```text
refusing to bind HTTP on 0.0.0.0:8765 without authentication.
Set LAB_MCP_HTTP_TOKEN or LAB_AUTH_MODE=oauth, or bind to 127.0.0.1 for local-only access.
```

Loopback addresses (`127.0.0.1`, `::1`, `[::1]`, `localhost`) are exempt.

## Middleware Stack (HTTP)

The HTTP server applies middleware in this order (outermost to innermost):

| Layer | Description |
|-------|-------------|
| `SetRequestId` | Generates a UUID v4 `x-request-id` for every request that lacks one. |
| `TraceLayer` | Tracing spans per request with method, path, status, and latency. |
| `PropagateRequestId` | Echoes `x-request-id` back in the response header. |
| `TimeoutLayer` | 30-second request timeout. Returns 504 on expiry. |
| `CompressionLayer` | gzip response compression. |
| `CorsLayer` | Explicit origin allowlist (see below). |
| Auth middleware | Bearer token and/or OAuth JWT validation. Applied to protected routes only. |

### CORS

CORS is configured with an explicit origin allowlist. It is not permissive by default.

Always allowed (loopback with common dev ports):

- `http://localhost`, `http://localhost:3000`, `http://localhost:5173`, `http://localhost:8080`
- `http://127.0.0.1`, `http://127.0.0.1:3000`, `http://127.0.0.1:5173`, `http://127.0.0.1:8080`
- `http://[::1]`

Additional origins via `LAB_CORS_ORIGINS` (comma-separated):

```bash
LAB_CORS_ORIGINS=https://lab.example.com,https://admin.example.com
```

Unparseable entries are logged as warnings and skipped.

Allowed methods: GET, POST, OPTIONS.
Allowed headers: `Authorization`, `Content-Type`, `x-request-id`.

## Route Layout

When HTTP transport is active, the server exposes:

| Path | Auth | Description |
|------|------|-------------|
| `/health` | no | Liveness probe |
| `/ready` | no | Readiness probe |
| `/.well-known/oauth-authorization-server` | no | Authorization-server metadata |
| `/.well-known/oauth-protected-resource` | no | RFC 9728 protected-resource metadata |
| `/jwks` | no | `lab` signing keys |
| `/register` | no | Bootstrap-secret-guarded dynamic client registration |
| `/authorize` | no | Authorization entrypoint |
| `/auth/google/callback` | no | Google callback endpoint |
| `/token` | no | Authorization-code and refresh-token exchange |
| `/v1/{service}` | yes | REST API (POST with action+params) |
| `/v1/{service}/actions` | yes | Action catalog (GET) |
| `/mcp` | yes | MCP streamable HTTP transport |

## Example: Local Development

```bash
# Bind to localhost, no auth needed when neither static bearer nor OAuth is configured
LAB_MCP_TRANSPORT=http lab serve
# → listening on 127.0.0.1:8765

curl http://localhost:8765/health
curl http://localhost:8765/v1/radarr -d '{"action":"help"}'
```

## Example: Network Deployment

```bash
# In ~/.lab/.env
LAB_MCP_TRANSPORT=http
LAB_MCP_HTTP_HOST=0.0.0.0
LAB_MCP_HTTP_PORT=8765
LAB_MCP_HTTP_TOKEN=$(openssl rand -hex 32)
LAB_PUBLIC_URL=https://lab.example.com

lab serve --transport http
```

```bash
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" \
     https://lab.example.com/v1/radarr \
     -d '{"action":"help"}'
```

## Related Docs

- [OAUTH.md](./OAUTH.md) — bearer vs OAuth mode, registration flow, and JWT validation
- [UPSTREAM.md](./UPSTREAM.md) — upstream MCP proxy
- [CONFIG.md](./CONFIG.md) — env var and config.toml loading
- [MCP.md](./MCP.md) — MCP protocol surface
- [RMCP.md](./RMCP.md) — RMCP SDK integration contract
