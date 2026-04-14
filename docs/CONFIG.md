# Configuration

Configuration is intentionally split between secrets and preferences.

## What goes where

| Category | Where | Examples |
|----------|-------|----------|
| Secrets | `~/.lab/.env` | `*_API_KEY`, `*_TOKEN`, `*_PASSWORD`, `LAB_MCP_HTTP_TOKEN` |
| URLs | `~/.lab/.env` | `*_URL`, `*_BASE_URL` |
| Everything else | `config.toml` | logging, MCP transport, CORS, admin flags, per-service prefs |

All `config.toml` values can still be overridden by env vars. Env always wins.

## Sources

Secrets and URLs live in:

- `~/.lab/.env`

Preferences live in (first found wins):

- `./config.toml` (repo/CWD override)
- `~/.lab/config.toml` (user-level, next to `.env`)
- `~/.config/lab/config.toml` (XDG-style fallback)

A reference file is provided at the repo root: `config.example.toml`.

## Load Order

Startup sequence:

1. `config.toml` (first found from the search order above)
2. Tracing init (using `[log]` section from config.toml)
3. `~/.lab/.env` (secrets + URLs via `dotenvy`)
4. `./env` from CWD (dev convenience, non-fatal if missing)

Value precedence at point of use (highest wins):

1. CLI flags (e.g. `--transport`, `--json`, `--port`)
2. Environment variables (whether from `.env` or the shell)
3. `config.toml`
4. Built-in defaults

## Config sections

### `[output]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `format` | `--json` flag | `"human"` | Default output format |

### `[log]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `filter` | `LAB_LOG` | `"lab=info,lab_apis=warn"` | Tracing filter directive |
| `format` | `LAB_LOG_FORMAT` | `"text"` | Log format: `"text"` or `"json"` |

### `[mcp]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `transport` | `LAB_MCP_TRANSPORT` | `"stdio"` | MCP transport: `"stdio"` or `"http"` |
| `host` | `LAB_MCP_HTTP_HOST` | `"127.0.0.1"` | HTTP bind address |
| `port` | `LAB_MCP_HTTP_PORT` | `8765` | HTTP bind port |

### `[api]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `cors_origins` | `LAB_CORS_ORIGINS` | `[]` | Additional CORS origins (loopback always included) |

### `[admin]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `enabled` | `LAB_ADMIN_ENABLED=1` | `false` | Enable the `lab_admin` MCP tool |

### `[services.tailscale]`

| Key | Env override | Default | Description |
|-----|-------------|---------|-------------|
| `tailnet` | `TAILSCALE_TAILNET` | `"-"` | Tailnet name (auto-detect) |

## Ownership

Config loading lives in `lab`, not `lab-apis`.

The SDK should not read files or ambient env automatically. It should receive explicit values when clients are constructed.

## Typed Config

Preferences are loaded into a typed config structure (`LabConfig`), not a stringly-typed map.

Rules:

- partial config files are valid
- validation happens once at load time
- service-specific config blocks use their own types under `[services.*]`

## Secret Handling

Credentials belong in env, not TOML.

Examples:

```env
RADARR_URL=http://localhost:7878
RADARR_API_KEY=abc123
PLEX_URL=http://localhost:32400
PLEX_TOKEN=xyz789
```

Rules:

- do not echo secrets in logs
- do not print secret env values in doctor or TUI prompts
- do not write credentials outside the designated env-management flows
- do not store secrets in config TOML

## Multi-Instance Naming

Multi-instance services use a predictable naming scheme.

Default instance:

```env
UNRAID_URL=https://tower.local/graphql
UNRAID_API_KEY=...
```

Named instance:

```env
UNRAID_SHART_URL=https://other.local/graphql
UNRAID_SHART_API_KEY=...
```

Rules:

- unlabeled keys define the `default` instance
- labeled keys define additional instances
- labels are derived from env, not hardcoded in source

## Default Instance Resolution

Resolution order:

1. explicit `default` instance
2. the sole configured instance
3. otherwise, require an explicit instance

## Future Migration Path

If env-based instance definitions become unwieldy, the project can later move instance metadata into TOML while still keeping secrets in env. The public CLI and MCP instance surface should remain stable across that migration.

## HTTP Auth Configuration

HTTP auth is mode-based.

- `LAB_AUTH_MODE=bearer` preserves the existing static bearer flow and still uses `LAB_MCP_HTTP_TOKEN`.
- `LAB_AUTH_MODE=oauth` enables the internal Google-backed authorization server and requires `LAB_PUBLIC_URL`.

Full details in [OAUTH.md](./OAUTH.md).

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LAB_AUTH_MODE` | no | `bearer` or `oauth`. Defaults to `bearer`. |
| `LAB_MCP_HTTP_TOKEN` | bearer mode only | Static bearer token for protected HTTP routes. |
| `LAB_PUBLIC_URL` | oauth mode | Public base URL for metadata, JWT issuer/audience, callback construction, and allowed-host derivation. |
| `LAB_AUTH_SQLITE_PATH` | no | Override path for the auth SQLite database. Defaults to `~/.lab/auth.db`. |
| `LAB_AUTH_KEY_PATH` | no | Override path for the persisted JWT signing key. Defaults to `~/.lab/auth-jwt.pem`. |
| `LAB_AUTH_BOOTSTRAP_SECRET` | oauth mode | Bootstrap bearer secret required for `/register`. |
| `LAB_GOOGLE_CLIENT_ID` | oauth mode | Google OAuth client ID. |
| `LAB_GOOGLE_CLIENT_SECRET` | oauth mode | Google OAuth client secret. |
| `LAB_GOOGLE_CALLBACK_PATH` | no | Callback path appended to `LAB_PUBLIC_URL`. Defaults to `/auth/google/callback`. |
| `LAB_GOOGLE_SCOPES` | no | Comma-separated Google scopes. Defaults to `openid,email,profile`. |

### config.toml

```toml
[auth]
mode = "oauth"
public_url = "https://lab.example.com"
bootstrap_secret = "set-via-env-in-real-deployments"
google_client_id = "google-client-id"
google_client_secret = "google-client-secret"
google_callback_path = "/auth/google/callback"
google_scopes = ["openid", "email", "profile"]
```

Environment variables override `[auth]` values field-by-field.

## Upstream MCP Servers

Lab can proxy tool calls and resource reads to upstream MCP servers.

Full details in [UPSTREAM.md](./UPSTREAM.md).

### config.toml

```toml
[[upstream]]
name = "remote-lab"
url = "https://lab2.example.com/mcp"
bearer_token_env = "LAB_UPSTREAM_TOKEN"
proxy_resources = true

[[upstream]]
name = "local-server"
command = "my-mcp-server"
args = ["--port", "5000"]
proxy_resources = false
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LAB_UPSTREAM_MAX_RESPONSE_BYTES` | 10485760 | Maximum response size from upstream servers. |
| (per `bearer_token_env`) | — | Bearer token for each upstream, named in config. |

## Transport and Session Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `LAB_MCP_TRANSPORT` | `stdio` | Transport: `stdio` or `http`. |
| `LAB_MCP_HTTP_HOST` | `127.0.0.1` | HTTP bind address. |
| `LAB_MCP_HTTP_PORT` | `8765` | HTTP bind port. |
| `LAB_MCP_HTTP_TOKEN` | — | Static bearer token for HTTP auth. |
| `LAB_MCP_SESSION_TTL_SECS` | `300` | MCP session keep-alive TTL (seconds). |
| `LAB_MCP_STATEFUL` | `true` | Whether to use stateful MCP sessions. |
| `LAB_MCP_ALLOWED_HOSTS` | — | Comma-separated hostnames for DNS rebinding protection. |
| `LAB_CORS_ORIGINS` | — | Comma-separated CORS origin allowlist. |

Full details in [TRANSPORT.md](./TRANSPORT.md).

## `.mcp.json` Environment

When `lab` is integrated into an MCP client config, the env file path should be explicit and stable so plugin installation and operator tooling update the same source of truth.
