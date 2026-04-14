# Sonarr API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/upstream-api/sonarr.openapi.json
**OpenAPI version:** 3.0.1
**API version:** 3.0.0
**Paths:** 162
**Servers:** {protocol}://{hostpath}
**Security schemes:** X-Api-Key

## Implementation Status

The Sonarr service is **fully onboarded**. The shared dispatch layer
(`crates/lab/src/dispatch/sonarr/`) is implemented with catalog, client, params, and
dispatch modules. The SDK client (`crates/lab-apis/src/sonarr/`) covers all priority
endpoints below.

### Surface wiring

| Surface | Status | Notes |
|---------|--------|-------|
| SDK (`lab-apis`) | ✅ | `SonarrClient` in `crates/lab-apis/src/sonarr/client.rs` |
| Dispatch layer | ✅ | `crates/lab/src/dispatch/sonarr/` — catalog, client, params, dispatch |
| MCP | ✅ | `crates/lab/src/mcp/services/sonarr.rs` — thin wrapper over dispatch layer |
| CLI | ✅ | `crates/lab/src/cli/sonarr.rs` — thin shim calling `dispatch::sonarr::dispatch` |
| API | ✅ | `crates/lab/src/api/services/sonarr.rs` — axum route calling `dispatch_with_client` |

### Implemented Actions

Complete list of 33 actions implemented in the catalog (`crates/lab/src/dispatch/sonarr/catalog.rs`):

| Action | SDK Method | Endpoint | Destructive | MCP | CLI | API |
|--------|-----------|---------|-------------|-----|-----|-----|
| `help` | - | - | No | ✅ | ✅ | ✅ |
| `schema` | - | - | No | ✅ | ✅ | ✅ |
| `series.list` | `series_list()` | GET /api/v3/series | No | ✅ | ✅ | ✅ |
| `series.get` | `series_get(id)` | GET /api/v3/series/{id} | No | ✅ | ✅ | ✅ |
| `series.lookup` | `series_lookup(query)` | GET /api/v3/series/lookup | No | ✅ | ✅ | ✅ |
| `series.add` | `series_add(req)` | POST /api/v3/series | No | ✅ | ✅ | ✅ |
| `series.delete` | `series_delete(id, delete_files)` | DELETE /api/v3/series/{id} | **Yes** | ✅ | ✅ | ✅ |
| `series.edit` | `series_edit(id, body)` | PUT /api/v3/series/{id} | No | ✅ | ✅ | ✅ |
| `episode.list` | `episode_list(series_id)` | GET /api/v3/episode | No | ✅ | ✅ | ✅ |
| `episode.get` | `episode_get(id)` | GET /api/v3/episode/{id} | No | ✅ | ✅ | ✅ |
| `episode.monitor` | `episode_monitor(episode_ids, monitored)` | PUT /api/v3/episode/monitor | No | ✅ | ✅ | ✅ |
| `queue.list` | `queue_list(query)` | GET /api/v3/queue | No | ✅ | ✅ | ✅ |
| `queue.delete` | `queue_delete(id, blocklist)` | DELETE /api/v3/queue/{id} | **Yes** | ✅ | ✅ | ✅ |
| `history.list` | `history_list(query)` | GET /api/v3/history | No | ✅ | ✅ | ✅ |
| `history.series` | `history_series(series_id)` | GET /api/v3/history/series | No | ✅ | ✅ | ✅ |
| `history.failed-retry` | `history_failed_retry(id)` | POST /api/v3/history/failed/{id} | No | ✅ | ✅ | ✅ |
| `wanted.list` | `wanted_missing(page, page_size)` | GET /api/v3/wanted/missing | No | ✅ | ✅ | ✅ |
| `wanted.cutoff` | `wanted_cutoff(page, page_size)` | GET /api/v3/wanted/cutoff | No | ✅ | ✅ | ✅ |
| `calendar.list` | `calendar_list(query)` | GET /api/v3/calendar | No | ✅ | ✅ | ✅ |
| `health` | `health()` | GET /api/v3/health | No | ✅ | ✅ | ✅ |
| `system.status` | `system_status()` | GET /api/v3/system/status | No | ✅ | ✅ | ✅ |
| `system.restart` | `system_restart()` | POST /api/v3/system/restart | **Yes** | ✅ | ✅ | ✅ |
| `system.backup` | `system_backup()` | GET /api/v3/system/backup | No | ✅ | ✅ | ✅ |
| `tag.list` | `tag_list()` | GET /api/v3/tag | No | ✅ | ✅ | ✅ |
| `tag.create` | `tag_create(label)` | POST /api/v3/tag | No | ✅ | ✅ | ✅ |
| `tag.delete` | `tag_delete(id)` | DELETE /api/v3/tag/{id} | **Yes** | ✅ | ✅ | ✅ |
| `rootfolder.list` | `rootfolder_list()` | GET /api/v3/rootfolder | No | ✅ | ✅ | ✅ |
| `qualityprofile.list` | `qualityprofile_list()` | GET /api/v3/qualityprofile | No | ✅ | ✅ | ✅ |
| `languageprofile.list` | `languageprofile_list()` | GET /api/v3/languageprofile | No | ✅ | ✅ | ✅ |
| `release.search` | `release_search(series_id, season_number)` | GET /api/v3/release | No | ✅ | ✅ | ✅ |
| `release.grab` | `release_grab(body)` | POST /api/v3/release | No | ✅ | ✅ | ✅ |
| `blocklist.list` | `blocklist_list()` | GET /api/v3/blocklist | No | ✅ | ✅ | ✅ |
| `blocklist.delete` | `blocklist_delete(id)` | DELETE /api/v3/blocklist/{id} | **Yes** | ✅ | ✅ | ✅ |
| `episodefile.delete` | `episodefile_delete(id)` | DELETE /api/v3/episodefile/{id} | **Yes** | ✅ | ✅ | ✅ |

Built-in actions `help` and `schema` are automatically available on every tool (handled in
`dispatch.rs` before the action match). All 31 service-specific actions are fully wired across
all three surfaces: MCP, CLI, and API.

### Action Parameters

#### Catalog Operations

**`series.list`** — no parameters. Returns all series in the library.

**`series.get`** — required: `id` (i64). Returns a single series by Sonarr ID.

**`series.lookup`** — required: `query` (string, e.g. "breaking bad" or "tvdb:81189"). 
Searches for series candidates by TVDB ID or search term.

**`series.add`** — required: `tvdb_id` (i64), `title` (string), `quality_profile_id` (i64),
`language_profile_id` (i64), `root_folder_path` (string). Optional: `monitored` (bool,
default true), `series_type` (string: standard|daily|anime, default: standard).

**`series.delete`** — required: `id` (i64). Optional: `delete_files` (bool, default false).
Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.
Destructive action.

**`series.edit`** — required: `id` (i64), `body` (object, full series resource from 
series.get then modify). Updates an existing series with a complete series resource body.

#### Episode Operations

**`episode.list`** — required: `series_id` (i64). Lists all episodes for a series.

**`episode.get`** — required: `id` (i64). Returns a single episode by ID.

**`episode.monitor`** — required: `episode_ids` (i64[]), `monitored` (bool).
Sets the monitored state for one or more episodes.

#### Queue Operations

**`queue.list`** — all optional: `page` (u32), `page_size` (u32), `series_id` (i64).
Lists the download queue with optional pagination and series filtering.

**`queue.delete`** — required: `id` (i64). Optional: `blocklist` (bool, default false).
Removes an item from the download queue. Destructive action.

#### History Operations

**`history.list`** — all optional: `page` (u32), `page_size` (u32), `series_id` (i64),
`episode_id` (i64). Lists download history with optional filtering.

**`history.series`** — required: `series_id` (i64). Lists history records for a specific series.

**`history.failed-retry`** — required: `id` (i64). Retries a failed download by history ID.

#### Wanted Operations

**`wanted.list`** — all optional: `page` (u32), `page_size` (u32).
Lists wanted/missing episodes with optional pagination.

**`wanted.cutoff`** — all optional: `page` (u32), `page_size` (u32).
Lists episodes that have not met their cutoff quality with optional pagination.

#### Calendar Operations

**`calendar.list`** — all optional: `start` (string, ISO 8601), `end` (string, ISO 8601),
`unmonitored` (bool, default false). Lists upcoming episode air dates within an optional 
date range, optionally including unmonitored episodes.

#### System & Health Operations

**`health`** — no parameters. Returns Sonarr health check results.

**`system.status`** — no parameters. Returns Sonarr system status and version.

**`system.restart`** — no parameters. Restarts the Sonarr application. Destructive action.
Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

**`system.backup`** — no parameters. Lists available system backup files.

#### Tag Operations

**`tag.list`** — no parameters. Lists all tags.

**`tag.create`** — required: `label` (string). Creates a new tag with the given label.

**`tag.delete`** — required: `id` (i64). Deletes a tag by ID. Destructive action.
Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

#### Profile Operations

**`rootfolder.list`** — no parameters. Lists all root folders.

**`qualityprofile.list`** — no parameters. Lists all quality profiles.

**`languageprofile.list`** — no parameters. Lists all language profiles.

#### Release Operations

**`release.search`** — all optional: `series_id` (i64), `season_number` (i32).
Searches for available releases for a series or season.

**`release.grab`** — required: `guid` (string, from release.search results).
Grabs a release by GUID and sends it to the download client.

#### Blocklist Operations

**`blocklist.list`** — no parameters. Lists all blocklisted releases.

**`blocklist.delete`** — required: `id` (i64). Removes a release from the blocklist by ID.
Destructive action. Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

#### Episode File Operations

**`episodefile.delete`** — required: `id` (i64). Deletes an episode file from disk by ID.
Destructive action. Requires `confirm: true` on the API surface; requires `-y`/`--yes` on the CLI.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired on this surface |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation status summary.

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
| DELETE | /api/v3/episode | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episode | `episode_list` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v3/episode | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episode/monitor | `episode_monitor` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/episode/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episode/{id} | `episode_get` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/episodefile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episodefile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/episodefile/{id} | `episodefile_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/extension | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/health | `health` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/history | `history_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/history/series | `history_series` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/history/failed/{id} | `history_failed_retry` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/languageprofile | `languageprofile_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/languageprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/log | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/log | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/log/files | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/notification/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/notificationschema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile | `qualityprofile_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/qualityprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue | `queue_list` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/queue | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/queue/{id} | `queue_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/release | `release_search` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/release | `release_grab` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/rootfolder | `rootfolder_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/rootfolder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series | `series_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/series | `series_add` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/series | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/series/{id} | `series_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/series/{id} | `series_get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v3/series/{id} | `series_edit` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/series/lookup | `series_lookup` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/tag | `tag_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/tag | `tag_create` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v3/tag/{id} | `tag_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/backup | `system_backup` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/system/status | `system_status` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v3/system/restart | `system_restart` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/wanted/cutoff | `wanted_cutoff` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v3/wanted/missing | `wanted_missing` | ✅ | ✅ | ✅ | ✅ |
