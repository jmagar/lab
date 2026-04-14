# Sonarr API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** docs/upstream-api/sonarr.openapi.json
**OpenAPI version:** 3.0.1
**API version:** 3.0.0
**Paths:** 162
**Servers:** {protocol}://{hostpath}
**Security schemes:** X-Api-Key

## Implementation Status

The Sonarr service is **fully onboarded**. The shared dispatch layer
(`crates/lab/src/dispatch/sonarr/`) is implemented with catalog, client, params, and
dispatch modules. The SDK client (`crates/lab-apis/src/sonarr/`) covers the priority
endpoints below.

### Surface wiring

| Surface | Status | Notes |
|---------|--------|-------|
| SDK (`lab-apis`) | ✅ | `SonarrClient` in `crates/lab-apis/src/sonarr/client.rs` |
| Dispatch layer | ✅ | `crates/lab/src/dispatch/sonarr/` — catalog, client, params, dispatch |
| MCP | ✅ | `crates/lab/src/mcp/services/sonarr.rs` — thin wrapper over dispatch layer |
| CLI | ✅ | `crates/lab/src/cli/sonarr.rs` — thin shim calling `dispatch::sonarr::dispatch` directly |
| API | ✅ | `crates/lab/src/api/services/sonarr.rs` — axum route calling `dispatch_with_client` |

### Implemented Actions

| Action | SDK Method | Endpoint | Destructive |
|--------|-----------|---------|-------------|
| `series.list` | `series_list()` | GET /api/v3/series | No |
| `series.get` | `series_get(id)` | GET /api/v3/series/{id} | No |
| `series.lookup` | `series_lookup(query)` | GET /api/v3/series/lookup | No |
| `series.add` | `series_add(req)` | POST /api/v3/series | No |
| `series.delete` | `series_delete(id, delete_files)` | DELETE /api/v3/series/{id} | **Yes** |
| `episode.list` | `episode_list(series_id)` | GET /api/v3/episode | No |
| `episode.get` | `episode_get(id)` | GET /api/v3/episode/{id} | No |
| `queue.list` | `queue_list(query)` | GET /api/v3/queue | No |
| `queue.delete` | `queue_delete(id, blocklist)` | DELETE /api/v3/queue/{id} | **Yes** |
| `history.list` | `history_list(query)` | GET /api/v3/history | No |
| `wanted.list` | `wanted_missing(page, page_size)` | GET /api/v3/wanted/missing | No |
| `calendar.list` | `calendar_list(query)` | GET /api/v3/calendar | No |
| `health` | `health()` | GET /api/v3/health | No |
| `system.status` | `system_status()` | GET /api/v3/system/status | No |
| `tag.list` | `tag_list()` | GET /api/v3/tag | No |
| `tag.create` | `tag_create(label)` | POST /api/v3/tag | No |
| `tag.delete` | `tag_delete(id)` | DELETE /api/v3/tag/{id} | **Yes** |
| `rootfolder.list` | `rootfolder_list()` | GET /api/v3/rootfolder | No |
| `qualityprofile.list` | `qualityprofile_list()` | GET /api/v3/qualityprofile | No |
| `languageprofile.list` | `languageprofile_list()` | GET /api/v3/languageprofile | No |
| `series.edit` | `series_edit(id, body)` | PUT /api/v3/series/{id} | No |
| `episode.monitor` | `episode_monitor(episode_ids, monitored)` | PUT /api/v3/episode/monitor | No |
| `wanted.cutoff` | `wanted_cutoff(page, page_size)` | GET /api/v3/wanted/cutoff | No |
| `release.search` | `release_search(series_id, season_number)` | GET /api/v3/release | No |
| `release.grab` | `release_grab(body)` | POST /api/v3/release | No |
| `history.series` | `history_series(series_id)` | GET /api/v3/history/series | No |
| `history.failed-retry` | `history_failed_retry(id)` | POST /api/v3/history/failed/{id} | No |
| `blocklist.list` | `blocklist_list()` | GET /api/v3/blocklist | No |
| `blocklist.delete` | `blocklist_delete(id)` | DELETE /api/v3/blocklist/{id} | **Yes** |
| `episodefile.delete` | `episodefile_delete(id)` | DELETE /api/v3/episodefile/{id} | **Yes** |
| `system.restart` | `system_restart()` | POST /api/v3/system/restart | **Yes** |
| `system.backup` | `system_backup()` | GET /api/v3/system/backup | No |

Built-in actions `help` and `schema` are also available on every tool (handled in
`dispatch.rs` before the action match).

### Action Parameters

**`series.add`** — required: `tvdb_id` (i64), `title` (string), `quality_profile_id` (i64),
`language_profile_id` (i64), `root_folder_path` (string). Optional: `monitored` (bool,
default true), `series_type` (string: standard|daily|anime, default: standard).

**`series.delete`** — required: `id` (i64). Optional: `delete_files` (bool, default false).
Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

**`series.edit`** — required: `id` (i64), `body` (object, full series resource from series.get).

**`episode.list`** — required: `series_id` (i64).

**`episode.get`** — required: `id` (i64).

**`episode.monitor`** — required: `episode_ids` (i64[]), `monitored` (bool).

**`queue.list`** — all optional: `page` (u32), `page_size` (u32), `series_id` (i64).

**`queue.delete`** — required: `id` (i64). Optional: `blocklist` (bool, default false).
Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

**`history.list`** — all optional: `page` (u32), `page_size` (u32), `series_id` (i64),
`episode_id` (i64).

**`history.series`** — required: `series_id` (i64).

**`history.failed-retry`** — required: `id` (i64).

**`wanted.list`** — all optional: `page` (u32), `page_size` (u32).

**`wanted.cutoff`** — all optional: `page` (u32), `page_size` (u32).

**`calendar.list`** — all optional: `start` (string, ISO 8601), `end` (string, ISO 8601),
`unmonitored` (bool).

**`release.search`** — all optional: `series_id` (i64), `season_number` (i32).

**`release.grab`** — required: `guid` (string).

**`blocklist.list`** — no parameters.

**`blocklist.delete`** — required: `id` (i64). Requires `confirm: true` / `-y`.

**`episodefile.delete`** — required: `id` (i64). Requires `confirm: true` / `-y`.

**`system.restart`** — no parameters. Requires `confirm: true` / `-y`.

**`system.backup`** — no parameters.

**`series.get`** — required: `id` (i64).

**`series.lookup`** — required: `query` (string, e.g. "breaking bad" or "tvdb:81189").

**`tag.create`** — required: `label` (string).

**`tag.delete`** — required: `id` (i64). Requires `confirm: true` / `-y`.

**`schema`** — required: `action` (string).

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Endpoint Inventory

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | / | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/autotagging | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/autotagging | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/autotagging/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/autotagging/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/autotagging/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/autotagging/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/blocklist | `blocklist_list` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/blocklist/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/blocklist/{id} | `blocklist_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/calendar | `calendar_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/calendar/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/command | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/command | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/command/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/command/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/host | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/host/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/host/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/importlist | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/importlist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/importlist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/indexer | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/indexer/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/indexer/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/mediamanagement | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/mediamanagement/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/mediamanagement/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/naming | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/naming/examples | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/naming/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/naming/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/ui | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/config/ui/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/config/ui/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/customfilter | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/customfilter | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/customformat | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/customformat | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/customformat/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/customformat/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/customformat/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/customformat/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/customformat/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/customformat/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/delayprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/delayprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/delayprofile/reorder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/delayprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/delayprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/delayprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/diskspace | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/downloadclient/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/downloadclient/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/downloadclient/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/downloadclient/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/downloadclient/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/downloadclient/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episode | `episode_list` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v3/episode/monitor | `episode_monitor` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/episode/{id} | `episode_get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v3/episode/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episodefile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/episodefile/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/episodefile/{id} | `episodefile_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem/mediafiles | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem/type | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/health | `health` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/history | `history_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/history/failed/{id} | `history_failed_retry` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/history/series | `history_series` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/history/since | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlist | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/importlist | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/importlist/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/importlist/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/importlist/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlist/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/importlist/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/importlist/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/importlist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/importlist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlistexclusion | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/importlistexclusion | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/importlistexclusion/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlistexclusion/paged | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/importlistexclusion/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/importlistexclusion/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/importlistexclusion/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/indexer | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/indexer | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/indexer/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/indexer/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/indexer/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/indexer/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/indexer/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/indexer/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/indexer/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/indexer/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/indexer/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/indexerflag | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/language | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/language/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/languageprofile | `languageprofile_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/languageprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/languageprofile/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/languageprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/languageprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/languageprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/localization | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/localization/language | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/localization/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log/file | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log/file/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log/file/update/{filename} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log/file/{filename} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/manualimport | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/manualimport | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/mediacover/{seriesId}/{filename} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/metadata | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/metadata | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/metadata/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/metadata/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/metadata/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/metadata/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/metadata/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/metadata/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/metadata/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notification/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/parse | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualitydefinition | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualitydefinition/limits | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/qualitydefinition/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualitydefinition/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/qualitydefinition/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile | `qualityprofile_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/qualityprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue | `queue_list` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/queue/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue/details | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/queue/grab/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/queue/grab/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue/status | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/queue/{id} | `queue_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/release | `release_search` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/release | `release_grab` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/release/push | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/releaseprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/releaseprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/releaseprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/releaseprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/releaseprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/remotepathmapping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/remotepathmapping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/remotepathmapping/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/remotepathmapping/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/remotepathmapping/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/rename | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/rootfolder | `rootfolder_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/rootfolder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/seasonpass | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series | `series_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/series | `series_add` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/series/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/series/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/series/import | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series/lookup | `series_lookup` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/series/{id} | `series_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/series/{id} | `series_get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v3/series/{id} | `series_edit` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/series/{id}/folder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/backup | `system_backup` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/system/backup/restore/upload | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/backup/restore/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/system/backup/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/restart | `system_restart` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/system/routes | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/routes/duplicate | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/shutdown | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/status | `system_status` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/system/task | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/task/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag | `tag_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/tag | `tag_create` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/tag/detail | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag/detail/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/tag/{id} | `tag_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/cutoff | `wanted_cutoff` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/wanted/cutoff/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/missing | `wanted_missing` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/wanted/missing/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /content/{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /feed/v3/calendar/sonarr.ics | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /logout | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| HEAD | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
