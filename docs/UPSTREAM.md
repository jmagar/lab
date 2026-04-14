# Upstream MCP Proxy

Lab can act as an MCP gateway, proxying tool calls and resource reads to upstream MCP servers. This lets a single lab instance aggregate tools from multiple MCP servers behind one authenticated endpoint.

The upstream pool lives in `crates/lab/src/dispatch/upstream/` — a shared dispatch module, not an MCP-specific module. Both the MCP and HTTP API surfaces can route to upstream servers.

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

### Config Validation

Validation runs before discovery. Invalid entries are skipped with a warning.

| Condition | Result |
|-----------|--------|
| Empty name | Skipped |
| Duplicate name | First wins, duplicates skipped with warning |
| Name contains `/`, `?`, or `#` | Skipped |
| URL not `http://` or `https://` | Skipped |
| URL uses bind-all address (`0.0.0.0`, `::`) | Skipped |
| Neither `url` nor `command` set | Skipped |

### Bearer Token

The `bearer_token_env` field names an environment variable — it does not contain the token directly. At connection time, the pool reads the env var and passes the token as an auth header for HTTP upstreams, or injects it into the child process environment for stdio upstreams.

If the named env var is not set, a warning is logged and the connection proceeds without auth.

## Discovery

At startup, lab connects to all configured upstreams in parallel. Each upstream gets a 15-second timeout for connection and tool discovery (`list_tools()`).

Failed upstreams are marked unhealthy. Healthy upstreams continue operating. A single failed upstream does not prevent others from connecting.

```
upstream discovery succeeded  upstream=remote-lab tool_count=12
upstream discovery failed     upstream=broken-server error="connection refused"
upstream discovery timed out  upstream=slow-server timeout_secs=15
```

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

- A successful call resets the upstream to healthy (0 failures).
- After the circuit breaker opens (3+ failures), the upstream is re-probed after 30 seconds.
- A successful re-probe resets the circuit breaker.

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

```
lab://upstream/{name}/{original_uri}
```

For example, if upstream `remote-lab` exposes a resource `lab://radarr/actions`, it appears as:

```
lab://upstream/remote-lab/lab://radarr/actions
```

### Operations

- `list_resources()` queries all resource-enabled upstreams and returns namespaced URIs.
- `read_resource()` strips the prefix, identifies the upstream by name, and forwards the read.

Failed resource listings from individual upstreams are logged as warnings. Other upstreams continue to serve.

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
