# qBittorrent API Coverage

**Last updated:** 2026-04-14
**Source spec:** docs/api-specs/qbittorrent.md
**Format:** dispatch layer + shared surfaces

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented (SDK + dispatch + MCP/CLI/API) |
| ⬜ | Not implemented yet |
| - | Not applicable |

## Summary

- **Auth:** qBittorrent uses cookie-based session auth (SID cookie). The lab binary reads `QBITTORRENT_URL` and `QBITTORRENT_SID` from env. The SID must be obtained externally by calling `POST /api/v2/auth/login` once and is passed via `Auth::Session { cookie }`.
- **All 32 dispatch actions are fully implemented** across all surfaces: SDK, dispatch layer, MCP, CLI, and API.
- **CLI:** thin dispatch shim (`lab qbittorrent <action> [--params <json>]`).
- **MCP:** exposes one tool: `qbittorrent`.
- **API:** route group at `POST /v1/qbittorrent`.
- **Surfaces:** All actions reachable via dispatch, MCP tool, CLI thin shim, and HTTP API handler.

## Action Inventory

| Action | SDK Method | Destructive | MCP | CLI | API | Params |
|--------|-----------|-------------|-----|-----|-----|--------|
| `help` | built-in | no | ✅ | ✅ | ✅ | — |
| `schema` | built-in | no | ✅ | ✅ | ✅ | `action: string` |
| `app.version` | `version()` | no | ✅ | ✅ | ✅ | — |
| `app.preferences` | `preferences()` | no | ✅ | ✅ | ✅ | — |
| `transfer.info` | `transfer_info()` | no | ✅ | ✅ | ✅ | — |
| `transfer.download.limit` | `set_download_limit(limit)` | no | ✅ | ✅ | ✅ | `limit: integer (bytes/s, 0=unlimited)` |
| `transfer.upload.limit` | `set_upload_limit(limit)` | no | ✅ | ✅ | ✅ | `limit: integer (bytes/s, 0=unlimited)` |
| `transfer.toggle-speed-limits` | `toggle_speed_limits()` | no | ✅ | ✅ | ✅ | — |
| `torrent.list` | `list_torrents(filter, category, limit)` | no | ✅ | ✅ | ✅ | `filter?: string`, `category?: string`, `limit?: integer` |
| `torrent.properties` | `torrent_properties(hash)` | no | ✅ | ✅ | ✅ | `hash: string` |
| `torrent.trackers` | `torrent_trackers(hash)` | no | ✅ | ✅ | ✅ | `hash: string` |
| `torrent.files` | `torrent_files(hash)` | no | ✅ | ✅ | ✅ | `hash: string` |
| `torrent.add` | `add_torrent(urls, savepath, category, tags)` | no | ✅ | ✅ | ✅ | `urls: string`, `savepath?: string`, `category?: string`, `tags?: string` |
| `torrent.pause` | `pause_torrents(hashes)` | no | ✅ | ✅ | ✅ | `hashes: string (pipe-sep or 'all')` |
| `torrent.resume` | `resume_torrents(hashes)` | no | ✅ | ✅ | ✅ | `hashes: string (pipe-sep or 'all')` |
| `torrent.delete` | `delete_torrents(hashes, delete_files)` | **yes** | ✅ | ✅ | ✅ | `hashes: string`, `delete_files?: bool` |
| `torrent.recheck` | `recheck_torrents(hashes)` | no | ✅ | ✅ | ✅ | `hashes: string (pipe-sep or 'all')` |
| `torrent.reannounce` | `reannounce(hashes)` | no | ✅ | ✅ | ✅ | `hashes: string (pipe-sep or 'all')` |
| `torrent.category.set` | `set_category(hashes, category)` | no | ✅ | ✅ | ✅ | `hashes: string`, `category: string` |
| `torrent.download.limit` | `set_torrent_download_limit(hashes, limit)` | no | ✅ | ✅ | ✅ | `hashes: string`, `limit: integer (bytes/s, 0=unlimited)` |
| `torrent.upload.limit` | `set_torrent_upload_limit(hashes, limit)` | no | ✅ | ✅ | ✅ | `hashes: string`, `limit: integer (bytes/s, 0=unlimited)` |
| `torrent.set-file-prio` | `set_file_priority(hash, id, priority)` | no | ✅ | ✅ | ✅ | `hash: string`, `id: string (pipe-sep indices)`, `priority: integer (0/1/6/7)` |
| `torrent.set-location` | `set_location(hashes, location)` | no | ✅ | ✅ | ✅ | `hashes: string`, `location: string` |
| `torrent.add-tags` | `add_tags(hashes, tags)` | no | ✅ | ✅ | ✅ | `hashes: string`, `tags: string (comma-sep)` |
| `torrent.remove-tags` | `remove_tags(hashes, tags)` | no | ✅ | ✅ | ✅ | `hashes: string`, `tags: string (comma-sep, empty=all)` |
| `torrent.set-share-limits` | `set_share_limits(hashes, ratio_limit, seeding_time_limit, inactive_seeding_time_limit)` | no | ✅ | ✅ | ✅ | `hashes: string`, `ratio_limit: number`, `seeding_time_limit: integer`, `inactive_seeding_time_limit: integer` |
| `category.list` | `categories()` | no | ✅ | ✅ | ✅ | — |
| `category.create` | `create_category(category, save_path)` | no | ✅ | ✅ | ✅ | `category: string`, `savepath?: string` |
| `category.edit` | `edit_category(category, save_path)` | no | ✅ | ✅ | ✅ | `category: string`, `savepath: string` |
| `log.list` | `log(last_known_id)` | no | ✅ | ✅ | ✅ | `last_known_id?: integer (-1=all)` |
| `sync.maindata` | `sync_maindata(rid)` | no | ✅ | ✅ | ✅ | `rid?: integer (0=full dump, pass returned rid for delta)` |

## Return Types

| Action | Returns |
|--------|---------|
| `help` | Action catalog (Catalog struct) |
| `schema` | Action schema (ActionSpec) |
| `app.version` | `{ "version": string }` |
| `app.preferences` | `Preferences` object |
| `transfer.info` | `TransferInfo` object (speeds, limits, DHT nodes, connection status) |
| `transfer.download.limit` | `{ "ok": true }` |
| `transfer.upload.limit` | `{ "ok": true }` |
| `transfer.toggle-speed-limits` | `{ "ok": true }` |
| `torrent.list` | `Torrent[]` |
| `torrent.properties` | `TorrentProperties` object |
| `torrent.trackers` | `Tracker[]` |
| `torrent.files` | `TorrentFile[]` |
| `torrent.add` | `{ "ok": true }` |
| `torrent.pause` | `{ "ok": true }` |
| `torrent.resume` | `{ "ok": true }` |
| `torrent.delete` | `{ "ok": true }` |
| `torrent.recheck` | `{ "ok": true }` |
| `torrent.reannounce` | `{ "ok": true }` |
| `torrent.category.set` | `{ "ok": true }` |
| `torrent.download.limit` | `{ "ok": true }` |
| `torrent.upload.limit` | `{ "ok": true }` |
| `torrent.set-file-prio` | `{ "ok": true }` |
| `torrent.set-location` | `{ "ok": true }` |
| `torrent.add-tags` | `{ "ok": true }` |
| `torrent.remove-tags` | `{ "ok": true }` |
| `torrent.set-share-limits` | `{ "ok": true }` |
| `category.list` | `Category[]` (values of the category map) |
| `category.create` | `{ "ok": true }` |
| `category.edit` | `{ "ok": true }` |
| `log.list` | `LogEntry[]` |
| `sync.maindata` | `SyncMaindata` object (full or delta; always contains `rid` for next call) |

## `torrent.list` Filter Values

`filter` accepts: `all`, `downloading`, `seeding`, `completed`, `paused`, `active`, `inactive`, `resumed`, `stalled`, `stalled_uploading`, `stalled_downloading`, `errored`.

## File Priority Values

`torrent.set-file-prio` `priority` accepts: `0` (do not download), `1` (normal), `6` (high), `7` (maximum).

## Share Limit Sentinel Values

For `torrent.set-share-limits` ratio and time limits: `-2` = use global setting, `-1` = no limit, `≥0` = explicit cap.

## `sync.maindata` Usage

- Pass `rid=0` (or omit) for the first call — returns a full state dump.
- The response always includes a `rid` field. Pass this value as `rid` on the next call to receive only changes (delta update).
- Callers must persist the `rid` between calls. Passing `rid=0` again triggers another full dump.

## Configuration

| Env Var | Required | Description |
|---------|----------|-------------|
| `QBITTORRENT_URL` | yes | Base URL, e.g. `http://qbittorrent:8080` |
| `QBITTORRENT_SID` | recommended | Pre-obtained SID cookie string, e.g. `SID=abc123` |

Note: If `QBITTORRENT_SID` is absent, the client is constructed with an empty cookie. Requests will fail until a valid SID is supplied.

## Implementation Architecture

### SDK Layer (`crates/lab-apis/src/qbittorrent/`)

- **`client.rs`**: `QbittorrentClient` struct with 32 async methods covering all operations. Uses `Auth::Session { cookie }` for cookie-based auth.
- **`types.rs`**: Request and response types (Torrent, Category, Preferences, TransferInfo, etc.). Serde-derived for JSON marshaling.
- **`error.rs`**: `QbittorrentError` wrapping `ApiError` from the core layer.

### Dispatch Layer (`crates/lab/src/dispatch/qbittorrent/`)

- **`catalog.rs`**: Single source of truth for all 32 actions. Each action declares name, description, param specs, destructive flag, and return type.
- **`client.rs`**: `require_client()` — resolves client from env vars or returns structured error. Called by dispatch entry points.
- **`params.rs`**: Helpers for param coercion (require_str, optional_i64, etc.).
- **`dispatch.rs`**: Two entry points:
  - `dispatch(action, params)` — called by MCP and CLI. Resolves client from env, delegates to `dispatch_with_client()`.
  - `dispatch_with_client(client, action, params)` — called by HTTP API with pre-built client from `AppState`. Routes 32 actions + built-in help/schema.

### Surface Layers

- **MCP** (`crates/lab/src/registry.rs`): Thin bridge that registers shared dispatch directly.
- **CLI** (`crates/lab/src/cli/qbittorrent.rs`): Thin shim using `run_action_command()` helper. Accepts `action` positional + `--params <json>` flag.
- **API** (`crates/lab/src/api/services/qbittorrent.rs`): Feature-gated route group. Handler extracts `ActionRequest`, looks up client from `AppState`, calls `handle_action()` helper with pre-built client.

### Destructive Action Handling

- `torrent.delete` is marked `destructive: true` in the catalog.
- **MCP**: Triggers elicitation flow — client must confirm before execution.
- **CLI**: Requires `-y` or `--yes` flag (or `--no-confirm`). Without it on a TTY, prompts interactively.
- **API**: Requires `"confirm": true` in the request body JSON object. Returns 400 with `kind: "confirmation_required"` if absent.

## Not Implemented

The following qBittorrent WebUI API v2 sections have no SDK or dispatch coverage:

- Authentication (`POST /api/v2/auth/login`, `POST /api/v2/auth/logout`)
- Sync (`/api/v2/sync/torrentPeers`)
- Torrent add by file upload (multipart form with .torrent file)
- Torrent trackers — add/remove trackers
- Torrent priority — queue position management
- Tag management — create/delete global tags (not per-torrent tags)
- RSS (experimental)
- Search
- WebSocket/event notifications
