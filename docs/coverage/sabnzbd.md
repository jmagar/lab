# SABnzbd API Coverage

**Last updated:** 2026-04-13
**Source spec:** docs/upstream-api/sabnzbd.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.

## Dispatch Actions

All actions are implemented end-to-end (SDK → dispatch → MCP → CLI → API).

| Action | SDK Method | Params | Returns | Destructive |
|--------|-----------|--------|---------|-------------|
| `help` | — (built-in) | — | Catalog | no |
| `schema` | — (built-in) | `action: string` (required) | Schema | no |
| `server.version` | `version()` | — | `{ version: string }` | no |
| `server.stats` | `server_stats()` | — | `ServerStats` | no |
| `server.warnings` | `warnings()` | — | `string[]` | no |
| `server.fullstatus` | `server_fullstatus()` | — | `object` | no |
| `queue.list` | `queue()` | — | `QueueResponse` | no |
| `queue.pause` | `pause()` | — | `{ paused: bool }` | no |
| `queue.resume` | `resume()` | — | `{ resumed: bool }` | no |
| `queue.speed.limit` | `set_speed_limit(kbps)` | `kbps: integer` (required) | `{ speed_limit_set: bool }` | no |
| `queue.delete` | `queue_delete(nzo_id)` | `nzo_id: string` (required) | `{ deleted: bool }` | **yes** |
| `queue.addurl` | `queue_addurl(url, cat, priority)` | `url: string` (required), `cat: string` (optional), `priority: string` (optional) | `object` | no |
| `queue.set-complete-action` | `queue_set_complete_action(action)` | `action: string` (required) | `{ complete_action_set: bool }` | no |
| `history.list` | `history(limit)` | `limit: integer` (optional) | `HistoryResponse` | no |
| `history.delete` | `history_delete(nzo_id)` | `nzo_id: string` (required) | `{ deleted: bool }` | **yes** |
| `history.purge` | `history_purge()` | — | `{ purged: bool }` | **yes** |
| `history.retry` | `history_retry(nzo_id)` | `nzo_id: string` (required) | `{ retried: bool }` | no |
| `history.retry-all` | `history_retry_all()` | — | `{ retried_all: bool }` | no |
| `category.list` | `category_list()` | — | `object` | no |
| `pp.pause` | `pp_pause()` | — | `{ pp_paused: bool }` | no |
| `pp.resume` | `pp_resume()` | — | `{ pp_resumed: bool }` | no |
| `rss.fetch-now` | `rss_fetch_now()` | — | `object` | no |
| `config.get` | `config_get()` | — | `object` (sensitive fields redacted) | no |

**Surface wiring:**
- MCP: `mcp/services/sabnzbd.rs` forwards to `dispatch::sabnzbd::dispatch()`
- CLI: `cli/sabnzbd.rs` calls `dispatch::sabnzbd::dispatch()` directly; supports `--yes`/`-y`, `--no-confirm`, and `--dry-run` flags for destructive actions
- API: `api/services/sabnzbd.rs` calls `dispatch::sabnzbd::dispatch_with_client()` with pre-built client from `AppState`

**Auth:** Requires `SABNZBD_URL` and `SABNZBD_API_KEY`. The API key is passed as a query parameter (`&apikey=<key>`), not a header — `Auth::None` is used for the underlying `HttpClient`.

**Security:** `config.get` calls `sanitize_config()` in `dispatch/sabnzbd/dispatch.rs` before returning. Redacted fields: top-level `api_key`, `servers[*].password`, `indexers[*].apikey`.

## Section Inventory

| Section | Key Operations |
|---------|----------------|
| Queue | queue, pause, resume, speedlimit, change_complete_action, sort, addurl, addfile, addlocalfile, delete, purge, switch (+8 more) |
| History | history, retry, retry_all, delete, mark_as_completed |
| Status | status, fullstatus, unblock_server, delete_orphan, add_orphan |
| Configuration | get_config, set_config, del_config, set_config_default |
| Information | version, auth, get_cats, get_scripts, server_stats, warnings, translate, showlog |
| System Control | See source file |

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
