# Configuration

Configuration is intentionally split between secrets and preferences.

## Sources

Secrets live in:

- `~/.lab/.env`

Preferences live in:

- `~/.config/lab/config.toml`

## Load Order

Config resolution should follow this order:

1. `~/.lab/.env`
2. `~/.config/lab/config.toml`
3. environment overrides
4. explicit CLI path overrides such as `--config`

## Ownership

Config loading lives in `lab`, not `lab-apis`.

The SDK should not read files or ambient env automatically. It should receive explicit values when clients are constructed.

## Typed Config

Preferences should be loaded into a typed config structure rather than a stringly-typed map.

Examples of typed settings:

- default output format
- default MCP port
- per-service defaults such as quality profile or root folder

Rules:

- partial config files are valid
- validation happens once at load time
- service-specific config blocks may use service newtypes

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

Lab can proxy tool calls and resource reads to upstream MCP servers through its MCP surface.

Full details in [UPSTREAM.md](./UPSTREAM.md).
Gateway mutation workflows live in [GATEWAY.md](./GATEWAY.md).

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

After these entries are configured, start `lab serve` as usual and point MCP clients at `lab`. The clients connect to `lab`; `lab` connects to the upstreams and merges their tools into its own MCP catalog.

The `gateway` management surface edits this same `[[upstream]]` array in `~/.config/lab/config.toml`. It never stores bearer-token values in TOML; only `bearer_token_env` references are persisted and returned.

If you change the env var named by `bearer_token_env`, call `gateway.reload` to rebuild the live pool and pick up the new token value.

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
