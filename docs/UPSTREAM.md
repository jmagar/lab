# Upstream MCP Proxy

Lab can act as an MCP gateway, proxying tool calls and resource reads to upstream MCP servers. This lets a single `lab` instance aggregate tools from multiple MCP servers behind one authenticated endpoint.

Upstream servers are first-class providers in the merged MCP tool catalog. After discovery, their tools appear in `list_tools()` beside built-in `lab` tools. Callers do not need a separate tool or namespace to invoke proxied upstream tools themselves.

`lab` also exposes a separate `gateway` management surface for editing and reloading upstream definitions. That management surface is documented in [GATEWAY.md](./GATEWAY.md).

The upstream pool lives in `crates/lab/src/dispatch/upstream/` because it is shared infrastructure. The runtime proxy path described in this document is wired into the MCP surface. The HTTP API now exposes `/v1/gateway` for gateway management, but it still does not proxy arbitrary upstream MCP tools.

## What Operators Configure

To proxy an upstream server through `lab`, you configure one or more `[[upstream]]` entries in `~/.config/lab/config.toml`, optionally provide bearer-token env vars in `~/.lab/.env`, then start `lab serve` normally.

`lab` will:

1. connect to every configured upstream at startup
2. run tool discovery against each upstream
3. merge discovered tools into its own MCP catalog
4. serve the combined catalog through whichever MCP transport you expose from `lab`

That means the client connects only to `lab`:

- `lab serve` for stdio clients such as Claude Desktop
- `lab serve --transport http` for streamable HTTP MCP clients

The client never connects directly to the upstreams once `lab` is acting as the gateway.

## Configuration

Upstream servers are configured in `config.toml` using `[[upstream]]` array entries.

### HTTP Upstream

```toml
[[upstream]]
name = "remote-lab"
url = "https://lab2.example.com/mcp"
bearer_token_env = "LAB_UPSTREAM_TOKEN"
proxy_resources = true
```

### Stdio Upstream

```toml
[[upstream]]
name = "local-server"
command = "my-mcp-server"
args = ["--port", "5000"]
proxy_resources = false
```

### Config Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Human-readable name. Must be non-empty, unique, and URI-safe (no `/`, `?`, `#`). |
| `url` | string | one of url/command | HTTP(S) URL of the upstream MCP server. |
| `command` | string | one of url/command | Command to run for stdio transport. |
| `args` | string[] | no | Arguments for the stdio command. |
| `bearer_token_env` | string | no | Name of an env var holding a bearer token. Not the token itself. |
| `proxy_resources` | bool | no | Whether to proxy resources from this upstream. Default: `false`. |

Exactly one of `url` or `command` must be set.

### Config File Locations

`lab` loads configuration from:

1. process environment
2. `~/.lab/.env`
3. `~/.config/lab/config.toml`

So a typical gateway setup looks like:

`~/.config/lab/config.toml`

```toml
[mcp]
transport = "http"
host = "127.0.0.1"
port = 8765

[[upstream]]
name = "remote-lab"
url = "https://lab2.example.com/mcp"
bearer_token_env = "REMOTE_LAB_TOKEN"
proxy_resources = true

[[upstream]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/srv/data"]
proxy_resources = false
```

`~/.lab/.env`

```bash
REMOTE_LAB_TOKEN=replace-me
LAB_MCP_HTTP_TOKEN=replace-this-too
```

### Config Validation

Validation runs before discovery. Invalid entries are skipped with a warning during startup discovery. The runtime `gateway` management surface rejects invalid mutations before writing them to disk.

| Condition | Result |
|-----------|--------|
| Empty name | Skipped |
| Duplicate name | Startup keeps the first and warns; runtime gateway mutations reject the write |
| Name contains `/`, `?`, or `#` | Skipped |
| URL not `http://` or `https://` | Skipped |
| URL uses bind-all address (`0.0.0.0`, `::`) | Skipped |
| Both `url` and `command` set | Skipped |
| Neither `url` nor `command` set | Skipped |

### Bearer Token

The `bearer_token_env` field names an environment variable — it does not contain the token directly. At connection time, the pool reads the env var and passes the token as an auth header for HTTP upstreams, or injects it into the child process environment for stdio upstreams.

If the named env var is not set, the connection proceeds without auth (HTTP upstreams log a warning; stdio upstreams currently skip injection silently).

Changing a bearer-token env var does not hot-apply by itself. Use `gateway.reload` when you want the live pool to re-read `bearer_token_env`.

## Discovery

At startup, lab connects to all configured upstreams in parallel. Each upstream gets a 15-second timeout for connection and tool discovery (`list_tools()`).

Failed upstreams are marked unhealthy. Healthy upstreams continue operating. A single failed upstream does not prevent others from connecting.

```text
upstream discovery succeeded  upstream=remote-lab tool_count=12
upstream discovery failed     upstream=broken-server error="connection refused"
upstream discovery timed out  upstream=slow-server timeout_secs=15
```

## How Routing Works

The combined catalog is exposed as one MCP server, but ownership is still resolved internally.

For each incoming MCP tool call:

1. `lab` checks whether the tool name belongs to a built-in local service
2. if not, it checks the discovered upstream tool map
3. if an upstream owns that tool name, the request is proxied there using the original MCP arguments
4. the upstream result is normalized into `lab`'s usual success/error envelope shape

This internal precedence rule does not make upstream tools second-class. It is just how collisions are resolved.

## Tool Collision Handling

When upstream tools are merged into the lab tool catalog:

1. **Built-in lab services always take precedence.** If an upstream exposes a tool named `radarr`, the upstream tool is silently dropped (with a warning logged).
2. **Cross-upstream duplicates: first discovered wins.** If two upstreams expose a tool named `my-tool`, the second is skipped with a warning.

Upstream tools appear alongside built-in tools in `list_tools()`. Callers do not need to know whether a tool is built-in or proxied.

## Circuit Breaker

Each upstream has independent health tracking.

| Constant | Value |
|----------|-------|
| `CIRCUIT_BREAKER_THRESHOLD` | 3 consecutive failures |
| `REPROBE_INTERVAL` | 30 seconds |

### State Transitions

- **Healthy** — upstream is routable. 0 consecutive failures.
- **Unhealthy (below threshold)** — upstream has 1-2 consecutive failures. Still routable and included in tool listings.
- **Unhealthy (at/above threshold)** — upstream has 3+ consecutive failures. Excluded from tool listings.

### What Counts as a Failure

- Connection errors
- Tool call errors (`is_error` responses)
- Dropped connections
- Response size cap exceeded

### Recovery

- A successful proxied call resets the upstream to healthy (0 failures).
- The code defines a `REPROBE_INTERVAL` of 30 seconds and tracks when an upstream became unhealthy.
- Automatic scheduled re-probing is not currently wired into the runtime. In practice, recovery happens when a later proxied call or resource request succeeds.

## Response Size Cap

Upstream responses are subject to a size cap to prevent oversized payloads from consuming memory or being forwarded to callers.

| Setting | Default |
|---------|---------|
| `LAB_UPSTREAM_MAX_RESPONSE_BYTES` | 10 MB (10,485,760 bytes) |

The check is **post-hoc** — rmcp materializes the full response in memory before lab can inspect it. The cap prevents forwarding oversized payloads to callers but cannot prevent the memory allocation itself. A streaming limit would require rmcp transport-level support.

The cap applies to both `call_tool` and `read_resource` responses.

## Resource Proxying

Resource proxying is opt-in per upstream via `proxy_resources = true`.

### URI Namespacing

Upstream resources are prefixed to avoid URI collisions with lab's own resources:

```text
lab://upstream/{name}/{original_uri}
```

For example, if upstream `remote-lab` exposes a resource `lab://radarr/actions`, it appears as:

```text
lab://upstream/remote-lab/lab://radarr/actions
```

### Operations

- `list_resources()` queries all resource-enabled upstreams and returns namespaced URIs.
- `read_resource()` strips the prefix, identifies the upstream by name, and forwards the read.

Failed resource listings from individual upstreams are logged as warnings. Other upstreams continue to serve.

## What Is Exposed Where

### MCP

The upstream gateway is active on both MCP transports exposed by `lab`:

- stdio
- streamable HTTP at `/mcp`

If an upstream tool is discovered successfully, MCP clients connected to `lab` can call it as a normal tool.

### HTTP API

The product HTTP API under `/v1/*` does not proxy arbitrary upstream MCP tools. It serves built-in `lab` routes plus `/v1/gateway` for gateway management.

Keep this distinction explicit in operator docs:

- use MCP when you want the upstream gateway behavior
- use `/v1/gateway` when you want to manage `[[upstream]]` entries over HTTP
- use the rest of `/v1/*` for `lab`'s built-in HTTP API surface

## End-to-End Setup

### 1. Configure upstreams

Add one or more `[[upstream]]` entries to `~/.config/lab/config.toml`.

### 2. Provide any required secrets

Set bearer-token env vars named by `bearer_token_env` in `~/.lab/.env` or the process environment.

### 3. Start `lab`

For local stdio clients:

```bash
lab serve
```

For network MCP clients:

```bash
lab serve --transport http
```

### 4. Point the client at `lab`, not the upstreams

Example `.mcp.json` for stdio:

```json
{
  "mcpServers": {
    "lab": {
      "command": "lab",
      "args": ["serve"]
    }
  }
}
```

Example HTTP MCP endpoint:

```text
https://lab.example.com/mcp
```

### 5. Verify discovery

Startup logs should include lines like:

```text
upstream discovery succeeded  upstream=remote-lab tool_count=12
```

Then an MCP client connected to `lab` should see the upstream tools in `list_tools()`.

## Operational Notes

- Upstream tool schemas are cached from discovery and reused for MCP tool metadata.
- Upstream calls preserve the original MCP argument payload rather than forcing it through `lab`'s `action` + `params` wrapper.
- Upstream errors are normalized into `lab` envelopes and usually surface as `upstream_error`, `network_error`, `server_error`, `decode_error`, or `internal_error`.
- Response-size limits are enforced after the upstream response is materialized in memory.

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LAB_UPSTREAM_MAX_RESPONSE_BYTES` | 10485760 | Maximum response size from upstream servers. |
| (per `bearer_token_env`) | — | Bearer token for each upstream, named in config. |

## Observability

Discovery events are logged at `INFO` (success) and `WARN` (failure/timeout).

Circuit breaker state changes are logged:

- `WARN` when the breaker opens (3+ failures).
- `INFO` when the breaker resets (successful call after failure).

Tool collision warnings are logged at `WARN`.

## Related Docs

- [CONFIG.md](./CONFIG.md) — `[[upstream]]` config section
- [MCP.md](./MCP.md) — upstream tool merging in MCP surface
- [ERRORS.md](./ERRORS.md) — `upstream_error` kind
- [TRANSPORT.md](./TRANSPORT.md) — HTTP transport setup
