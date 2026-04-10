# Unraid Coverage

**Last updated:** 2026-04-10
**Source spec:** GraphQL introspection against tootie (10-1-0-2:31337/graphql)
**Auth:** `X-API-Key` header

## Implementation Status

| Surface | Status |
|---------|--------|
| SDK (lab-apis) | ✓ Complete |
| CLI shim | ✓ Complete (Tier-2) |
| MCP dispatcher | ✓ Complete |
| API handler | ✓ Complete |
| Health check | ✓ Complete |
| TUI metadata | ✓ Complete |

## Action Coverage

| Action | Method | Destructive |
|--------|--------|-------------|
| `system.info` | `GET info { os cpu system versions }` | No |
| `system.metrics` | `GET metrics { cpu memory }` | No |
| `system.array` | `GET array { state disks parities caches }` | No |
| `system.online` | `GET online` | No |
| `docker.list` | `GET docker { containers }` | No |
| `docker.start` | `mutation docker { start(id) }` | Yes |
| `docker.stop` | `mutation docker { stop(id) }` | Yes |
| `docker.restart` | stop + start (client-side, no native mutation) | Yes |
| `disk.list` | `GET disks { id name device vendor size type smartStatus temperature serialNum }` | No |

## Files

| File | Purpose |
|------|---------|
| `crates/lab-apis/src/unraid.rs` | Module declaration, META, ServiceClient impl |
| `crates/lab-apis/src/unraid/client.rs` | UnraidClient — 9 async methods |
| `crates/lab-apis/src/unraid/types.rs` | Request/response types (serde) |
| `crates/lab-apis/src/unraid/error.rs` | UnraidError (wraps ApiError) |
| `crates/lab/src/dispatch/unraid.rs` | Dispatch entrypoint (re-exports) |
| `crates/lab/src/dispatch/unraid/catalog.rs` | ACTIONS — 9 actions, 3 destructive |
| `crates/lab/src/dispatch/unraid/client.rs` | client_from_env(), require_client() |
| `crates/lab/src/dispatch/unraid/dispatch.rs` | dispatch() + dispatch_with_client() |
| `crates/lab/src/dispatch/unraid/params.rs` | require_id() param extractor |
| `crates/lab/src/cli/unraid.rs` | CLI shim (Tier-2: action + params passthrough) |
| `crates/lab/src/mcp/services/unraid.rs` | MCP thin shim → dispatch |
| `crates/lab/src/api/services/unraid.rs` | Axum route group → dispatch |
| `crates/lab/src/cli/health.rs` | unraid_row() health probe |
| `crates/lab/src/tui/metadata.rs` | TUI health check block |

## Known Limitations

- No in-process rate limiting — Unraid enforces ~100 req/10s; callers must stay within bounds.
- `docker.restart` is implemented as sequential stop + start (no native restart mutation in schema).
- Tier-2 CLI only (raw action + params); typed Tier-1 subcommands deferred.
- Subscriptions (17 available via graphql-transport-ws) deferred to a separate epic.
