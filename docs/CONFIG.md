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

## `.mcp.json` Environment

When `lab` is integrated into an MCP client config, the env file path should be explicit and stable so plugin installation and operator tooling update the same source of truth.
