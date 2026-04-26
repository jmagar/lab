# SABnzbd API Coverage

**Last updated:** 2026-04-14
**Source spec:** docs/upstream-api/sabnzbd.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.
- **23 actions implemented end-to-end** across SDK, dispatch layer, MCP, CLI, and API surfaces.

## Dispatch Actions

All actions are implemented end-to-end (SDK → dispatch → MCP → CLI → API).

| Action | SDK Method | Params | Returns | Destructive | MCP | CLI | API |
|--------|-----------|--------|---------|-------------|-----|-----|-----|
| `help` | — (built-in) | — | Catalog | no | ✓ | ✓ | ✓ |
| `schema` | — (built-in) | `action: string` (required) | Schema | no | ✓ | ✓ | ✓ |
| `server.version` | `version()` | — | `{ version: string }` | no | ✓ | ✓ | ✓ |
| `server.stats` | `server_stats()` | — | `ServerStats` | no | ✓ | ✓ | ✓ |
| `server.warnings` | `warnings()` | — | `string[]` | no | ✓ | ✓ | ✓ |
| `server.fullstatus` | `server_fullstatus()` | — | `object` | no | ✓ | ✓ | ✓ |
| `queue.list` | `queue()` | — | `QueueResponse` | no | ✓ | ✓ | ✓ |
| `queue.pause` | `pause()` | — | `{ paused: bool }` | no | ✓ | ✓ | ✓ |
| `queue.resume` | `resume()` | — | `{ resumed: bool }` | no | ✓ | ✓ | ✓ |
| `queue.speed.limit` | `set_speed_limit(kbps)` | `kbps: integer` (required) | `{ speed_limit_set: bool }` | no | ✓ | ✓ | ✓ |
| `queue.delete` | `queue_delete(nzo_id)` | `nzo_id: string` (required) | `{ deleted: bool }` | **yes** | ✓ | ✓ | ✓ |
| `queue.addurl` | `queue_addurl(url, cat, priority)` | `url: string` (required), `cat: string` (optional), `priority: string` (optional) | `object` | no | ✓ | ✓ | ✓ |
| `queue.set-complete-action` | `queue_set_complete_action(action)` | `action: string` (required) | `{ complete_action_set: bool }` | no | ✓ | ✓ | ✓ |
| `history.list` | `history(limit)` | `limit: integer` (optional) | `HistoryResponse` | no | ✓ | ✓ | ✓ |
| `history.delete` | `history_delete(nzo_id)` | `nzo_id: string` (required) | `{ deleted: bool }` | **yes** | ✓ | ✓ | ✓ |
| `history.purge` | `history_purge()` | — | `{ purged: bool }` | **yes** | ✓ | ✓ | ✓ |
| `history.retry` | `history_retry(nzo_id)` | `nzo_id: string` (required) | `{ retried: bool }` | no | ✓ | ✓ | ✓ |
| `history.retry-all` | `history_retry_all()` | — | `{ retried_all: bool }` | no | ✓ | ✓ | ✓ |
| `category.list` | `category_list()` | — | `object` | no | ✓ | ✓ | ✓ |
| `pp.pause` | `pp_pause()` | — | `{ pp_paused: bool }` | no | ✓ | ✓ | ✓ |
| `pp.resume` | `pp_resume()` | — | `{ pp_resumed: bool }` | no | ✓ | ✓ | ✓ |
| `rss.fetch-now` | `rss_fetch_now()` | — | `object` | no | ✓ | ✓ | ✓ |
| `config.get` | `config_get()` | — | `object` (sensitive fields redacted) | no | ✓ | ✓ | ✓ |

## Implementation Surface Details

**Dispatch architecture:** Shared `crates/lab/src/dispatch/sabnzbd/` layer owns all action routing, param validation, and client resolution. This layer is called by three independent adapters:

- **MCP:** `crates/lab/src/registry.rs` — thin bridge, registers `dispatch::sabnzbd::dispatch()` directly
- **CLI:** `crates/lab/src/cli/sabnzbd.rs` — thin shim, calls `dispatch::sabnzbd::dispatch()` with support for `--yes`/`-y`, `--no-confirm`, and `--dry-run` flags for destructive actions
- **API:** `crates/lab/src/api/services/sabnzbd.rs` — calls `dispatch::sabnzbd::dispatch_with_client()` with pre-built client from `AppState`

**Client resolution:**

- `dispatch/sabnzbd/client.rs` handles env var lookup: `SABNZBD_URL` and `SABNZBD_API_KEY`
- `client_from_env()` is called by `AppState` at startup (optional client, non-fatal if absent)
- `require_client()` is the MCP/CLI fallback when AppState is unavailable
- Returns structured `internal_error` if config is missing or client construction fails

**Auth:** Requires `SABNZBD_URL` and `SABNZBD_API_KEY`. The API key is passed as a query parameter (`&apikey=<key>`), not a header — the underlying `HttpClient` uses `Auth::None`.

**Security:** `config.get` calls `sanitize_config()` in `dispatch/sabnzbd/dispatch.rs` before returning. Redacted fields:
- top-level `api_key`
- `servers[*].password`
- `indexers[*].apikey`

## Destructive Actions

Three operations require confirmation on destructive transports:

| Action | Effect | Confirmation |
|--------|--------|--------------|
| `queue.delete` | Delete a queue item by NZO ID | CLI: `-y`/`--yes`; API: `"confirm": true` in params |
| `history.delete` | Delete a single history item by NZO ID | CLI: `-y`/`--yes`; API: `"confirm": true` in params |
| `history.purge` | Delete all completed history items | CLI: `-y`/`--yes`; API: `"confirm": true` in params |

## Section Inventory

| Section | Key Operations |
|---------|----------------|
| Queue | list, pause, resume, speed.limit, delete, addurl, set-complete-action |
| History | list, delete, purge, retry, retry-all |
| Status | fullstatus, stats, warnings, version |
| Categories | list |
| Post-Processing | pause, resume |
| RSS | fetch-now |
| Configuration | get (with field redaction) |

## Implemented vs. Upstream

| Upstream Mode | Implemented | Action |
|---------------|-------------|--------|
| `version` | ✅ | `server.version` |
| `queue` (list) | ✅ | `queue.list` |
| `queue` + `name=delete` | ✅ | `queue.delete` |
| `queue` + `name=addurl` | ✅ | `queue.addurl` |
| `queue` + `name=change_complete_action` | ✅ | `queue.set-complete-action` |
| `history` (list) | ✅ | `history.list` |
| `history` + `name=delete` (single) | ✅ | `history.delete` |
| `history` + `name=delete` + `value=all` | ✅ | `history.purge` |
| `history` + `name=retry` | ✅ | `history.retry` |
| `history` + `name=retry_all` | ✅ | `history.retry-all` |
| `server_stats` | ✅ | `server.stats` |
| `warnings` | ✅ | `server.warnings` |
| `fullstatus` | ✅ | `server.fullstatus` |
| `pause` | ✅ | `queue.pause` |
| `resume` | ✅ | `queue.resume` |
| `config` + `name=speedlimit` | ✅ | `queue.speed.limit` |
| `get_cats` | ✅ | `category.list` |
| `pause_pp` | ✅ | `pp.pause` |
| `resume_pp` | ✅ | `pp.resume` |
| `rss_now` | ✅ | `rss.fetch-now` |
| `get_config` | ✅ | `config.get` (api_key, server passwords, indexer apikeys redacted) |
| `queue` + `name=sort` | ⬜ | — |
| `queue` + `name=switch` | ⬜ | — |
| `status` | ⬜ | — |
| `set_config` | ⬜ | — |
| `get_scripts` | ⬜ | — |
| All other upstream modes | ⬜ | — |
