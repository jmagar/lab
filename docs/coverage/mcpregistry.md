# MCP Registry API Coverage

**Last updated:** 2026-04-22
**Upstream spec:** `docs/upstream-api/mcp-registry.yaml`
**Upstream base URL (default):** `https://registry.modelcontextprotocol.io`
**Override:** `MCPREGISTRY_URL`

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet |
| — | Not applicable |

## SDK Surface (`crates/lab-apis/src/mcpregistry/client.rs`)

| Method | Endpoint | SDK Method | Impl |
|--------|----------|------------|------|
| GET | /v0.1/servers | `list_servers()` | ✅ |
| GET | /v0.1/servers/{name}/versions | `list_versions()` | ✅ |
| GET | /v0.1/servers/{name}/versions/{version} | `get_server()` | ✅ |
| POST | /v0.1/validate | `validate()` | ✅ |
| GET | /v0.1/health | `health_probe()` (internal) | ✅ |

Notes:

- All endpoints are unauthenticated.
- Custom `reqwest::Client`: 20 s request timeout, 5 s connect, no redirect following (SSRF protection).
- Path segments are percent-encoded via `PATH_SEGMENT` set so reverse-DNS names
  (`io.github.user/my-server`) round-trip correctly.

## Dispatch Actions (`crates/lab/src/dispatch/mcpregistry/catalog.rs`)

The service exposes one MCP tool: `mcpregistry`. Action shape: `action + params` dispatch.

| Action | Params | Destructive | Returns |
|--------|--------|-------------|---------|
| `help` | none | No | Catalog |
| `schema` | `action: string` (required) | No | Schema |
| `config` | none | No | RegistryConfig |
| `server.list` | `search?`, `owner?`, `limit?`, `cursor?`, `version?`, `updated_since?`, `sort_by?`, `order?` | No | ServerListResponse |
| `server.get` | `name: string` (required) | No | ServerResponse |
| `server.versions` | `name: string` (required) | No | ServerListResponse |
| `server.install` | (see catalog) | **Yes** | InstallResult |
| `server.uninstall` | (see catalog) | **Yes** | UninstallResult |
| `server.validate` | (see catalog) | No | ValidationResult |
| `sync` | none | No | SyncResult |

### `server.list` — search and owner

Callers can filter by either full-text substring or GitHub namespace:

```jsonc
mcpregistry({ "action": "server.list", "params": { "search": "postgres" } })
mcpregistry({ "action": "server.list", "params": { "owner": "modelcontextprotocol" } })
```

Resolution rules (`crate::dispatch::mcpregistry::resolve_search_for_rest`):

1. Explicit `search` wins if present — `owner` is silently ignored.
2. `owner` is trimmed, lowercased, and expanded to `search = "io.github.{owner}/"`.
3. Invalid `owner` (empty, contains `/`, or contains whitespace) returns
   `invalid_param` rather than falling through to an unfiltered list.
4. `owner` does not match non-GitHub publishers (`io.gitlab.*`, custom namespaces).
   Use `search` directly for those.

The same resolver is used by the `/v0.1/servers` GET surface — the two paths have
identical filtering semantics.

### `server.list` — `/v1` upstream vs `/v0.1/servers` store

`server.list` calls the upstream registry directly (the `/v1` surface). Sort
operates within the current page only. For full-dataset sort and offline
availability, use the `GET /v0.1/servers` store endpoint described below.

## Surface Coverage

| Action | MCP | CLI | API (`/v1/mcpregistry`) | REST (`/v0.1/servers`) |
|--------|-----|-----|------|------|
| `server.list` | ✅ | ✅ | ✅ | ✅ (search + owner) |
| `server.get` | ✅ | ✅ | ✅ | ✅ (per-name GET) |
| `server.versions` | ✅ | ✅ | ✅ | ✅ |
| `server.install` | ✅ | ✅ | ✅ | — |
| `server.uninstall` | ✅ | ✅ | ✅ | — |
| `server.validate` | ✅ | ✅ | ✅ | — |
| `config` | ✅ | ✅ | ✅ | — |
| `sync` | ✅ | ✅ | ✅ | — |

### CLI (`crates/lab/src/cli/mcpregistry.rs`)

Tier-2 shim: `lab mcpregistry <action> [--params '<json>']`.

### MCP (`crates/lab/src/mcp/services/mcpregistry.rs`)

Thin bridge delegating to `crate::dispatch::mcpregistry::dispatch()`. One MCP tool `mcpregistry`.

### API — two mount points

The service exposes **two** HTTP surfaces:

1. **`POST /v1/mcpregistry`** — action+params dispatch, mirrors MCP exactly.
   Handler: `crates/lab/src/api/services/mcpregistry.rs`.
2. **`GET /v0.1/servers/*`** — REST wire-compatible with the upstream
   MCP Registry v0.1 spec. Handler: `crates/lab/src/api/services/registry_v01.rs`.

The REST surface backs the `lab://mcpregistry/…` UI and any consumer expecting the
upstream shape. It reads from the local SQLite registry store (populated by `sync`),
not the upstream — so it survives upstream outages and supports richer sort semantics.

| REST endpoint | Backing store | Query params |
|---------------|---------------|--------------|
| `GET /v0.1/servers` | `RegistryStore::list_servers` | `search`, `owner`, `cursor`, `limit`, `include_deleted` |
| `GET /v0.1/servers/:name/versions` | `RegistryStore::list_versions` | — |
| `GET /v0.1/servers/:name/versions/:version` | `RegistryStore::get_server` | — |

The REST endpoints require bearer auth (same token as the rest of the HTTP API).

## Client Construction

`crates/lab/src/dispatch/mcpregistry/client.rs`:

- `require_client()` — builds a client from `MCPREGISTRY_URL` (default
  `https://registry.modelcontextprotocol.io`). No auth required.
- `client_from_env()` — `Option`-returning variant for `AppState` startup.

## Config

| Env var | Required | Purpose |
|---------|----------|---------|
| `MCPREGISTRY_URL` | No | Override the default upstream base URL |
| `LAB_REGISTRY_DB` | No | SQLite path for the local registry store (default: `~/.lab/registry.db`) |

## Error Kinds

Standard SDK kinds (`auth_failed`, `not_found`, `rate_limited`, `invalid_param`,
`missing_param`, `internal_error`, `server_error`) apply. Two kinds are
registry-specific and documented in `docs/ERRORS.md`:

- `no_remote_transport` — `server.install` on a stdio-only server
- `ssrf_blocked` — registry-sourced URL resolves to a private/loopback/link-local host

Both use `ToolError::Sdk { sdk_kind, message }`; HTTP 422.

Additionally, the REST surface returns `sync_in_progress` (HTTP 503) when the
store is still initializing.

## SSRF Protection

Any URL flowing from the registry into the gateway upstream layer is validated by
`crate::dispatch::mcpregistry::validate_registry_url` before use. It rejects:

- non-HTTPS schemes (including `http`, `file`, `data`, `ftp`)
- hosts resolving to RFC1918, loopback, link-local, or ULA addresses
- hosts with raw IP literals that bypass DNS

`server.install` always runs validation before adding a remote as a gateway upstream.
