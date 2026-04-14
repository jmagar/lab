# dispatch/upstream/ — Upstream MCP Proxy Pool

Surface-neutral upstream MCP server proxy. Manages connections to external MCP servers (HTTP or stdio), discovers their tools, and routes `call_tool` / `read_resource` requests.

## Why dispatch/, not mcp/

Both the MCP surface and the HTTP API surface need access to `UpstreamPool`. The layer contract forbids `api -> mcp` dependencies, so shared types must live in the dispatch layer.

Dependency direction:

- `api -> dispatch/upstream`
- `mcp -> dispatch/upstream`
- `cli -> dispatch/upstream`

## Files

| File | Purpose |
|------|---------|
| `upstream.rs` | Module entrypoint. |
| `pool.rs` | `UpstreamPool` — connection management, discovery, circuit breaker, tool/resource proxying. |
| `types.rs` | `UpstreamEntry`, `UpstreamTool`, `UpstreamHealth`, `UpstreamConnection` types and constants. |

## Key Types

- `UpstreamPool` — holds live connections and discovered tool catalogs. Cloneable (Arc internals).
- `UpstreamEntry` — snapshot of a single upstream: name, tools, health state.
- `UpstreamTool` — a discovered tool with its cached input schema and owning upstream name.
- `UpstreamHealth` — `Healthy` or `Unhealthy { consecutive_failures }`.
- `UpstreamConnection` — a live rmcp `Peer<RoleClient>` with its owning `RunningService`.

## Constants

| Constant | Value | Location |
|----------|-------|----------|
| `CIRCUIT_BREAKER_THRESHOLD` | 3 | `types.rs` |
| `REPROBE_INTERVAL` | 30 seconds | `types.rs` |
| `DISCOVERY_TIMEOUT` | 15 seconds | `pool.rs` |
| `DEFAULT_MAX_RESPONSE_BYTES` | 10 MB | `pool.rs` |

## Rules

- Do not read env vars outside `pool.rs::max_response_bytes()` and the connection functions.
- Do not import MCP-specific types (envelopes, registry) from `mcp/`.
- Do not import API-specific types (router, state) from `api/`.
- The pool is constructed in `cli/serve.rs` and injected into `AppState` and `LabMcpServer`.
- Circuit breaker state is internal to the pool. Surfaces call `record_failure()` and `record_success()`.
