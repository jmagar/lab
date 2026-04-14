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

## OAuth Configuration

OAuth 2.1 resource server configuration. Lab validates JWT tokens issued by an external authorization server.

Full details in [OAUTH.md](./OAUTH.md).

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LAB_OAUTH_ISSUER` | when OAuth enabled | OIDC issuer URL. Must use HTTPS. |
| `LAB_OAUTH_AUDIENCE` | when OAuth enabled | Expected `aud` claim (RFC 8707). |
| `LAB_OAUTH_CLIENT_ID` | no | Optional `azp` claim validation. |
| `LAB_RESOURCE_URL` | when OAuth enabled | Public URL of this lab instance. |

### config.toml

```toml
[oauth]
issuer = "https://auth.example.com"
audience = "https://lab.example.com"
client_id = "optional-client-id"
resource_url = "https://lab.example.com"
```

Env vars override config.toml for every field.

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
