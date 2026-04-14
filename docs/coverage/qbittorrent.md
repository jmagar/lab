# qBittorrent API Coverage

**Last updated:** 2026-04-13
**Source spec:** docs/upstream-api/qbittorrent.md
**Format:** hand-scraped reference

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented (SDK + dispatch + MCP/CLI/API) |
| ⬜ | Not implemented yet |
| - | Not applicable |

## Summary

- Auth: qBittorrent uses cookie-based session auth (SID cookie). The lab binary reads `QBITTORRENT_URL` and `QBITTORRENT_SID` from env. The SID must be obtained externally by calling `POST /api/v2/auth/login` once and is passed via `Auth::Session { cookie }`.
- All 29 dispatch actions are reachable via CLI, MCP, and API.
- CLI is a thin dispatch shim (`lab qbittorrent <action> [--params <json>]`).
- MCP exposes one tool: `qbittorrent`.
- API route: `POST /v1/qbittorrent`.

## Action Inventory

| Action | SDK Method | Destructive | CLI | MCP | API | Params |
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
| `categories.list` | `categories()` | no | ✅ | ✅ | ✅ | — |
| `category.create` | `create_category(category, save_path)` | no | ✅ | ✅ | ✅ | `category: string`, `savepath?: string` |
| `category.edit` | `edit_category(category, save_path)` | no | ✅ | ✅ | ✅ | `category: string`, `savepath: string` |
| `log.list` | `log(last_known_id)` | no | ✅ | ✅ | ✅ | `last_known_id?: integer (-1=all)` |
| `sync.maindata` | `sync_maindata(rid)` | no | ✅ | ✅ | ✅ | `rid?: integer (0=full dump, pass returned rid for delta)` |

## Return Types

| Action | Returns |
|--------|---------|
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
| `categories.list` | `Category[]` (values of the category map) |
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
