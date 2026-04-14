# Radarr API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** `docs/upstream-api/radarr.openapi.json` (164 unique paths, 238 operations)
**SDK sub-modules:** `crates/lab-apis/src/radarr/client/` (21 modules, 54 methods)
**Dispatch module:** `crates/lab/src/dispatch/radarr/` (catalog.rs, dispatch.rs, system.rs, movies.rs, queue.rs, calendar.rs, commands.rs, history.rs, config.rs, wanted.rs, customformat.rs) — 51 actions + built-in `help` + `schema`
**MCP actions:** `crates/lab/src/mcp/services/radarr.rs` (thin adapter over dispatch layer)
**CLI subcommands:** `crates/lab/src/cli/radarr.rs` (39 typed subcommands — Tier-1 pattern)
**API handler:** `crates/lab/src/api/services/radarr.rs` (single POST route, dispatches via shared dispatch layer)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented / confirmed working |
| ⬜ | Not yet implemented, or wired but not live-tested |
| — | Not applicable |

**Column definitions:**
1. **Impl** — SDK client method exists in `lab-apis`
2. **MCP** — Action wired in MCP dispatcher (dispatch code confirmed present; ✅ requires live-test confirmation)
3. **CLI** — Typed subcommand wired in CLI (confirmed present; ✅ requires live-test confirmation)
4. **API** — HTTP POST `/v1/radarr` action dispatch wired (handler confirmed present; ✅ requires live-test confirmation)

> ⚠️ **All 39 dispatch actions are fully wired through MCP, CLI, and API surfaces.** The ⬜ symbol in MCP/CLI/API columns means the handler exists but has not been live-tested against a running Radarr instance. CI test pass alone does not earn ✅. A cell turns ✅ only after manual testing confirms correct end-to-end behavior.

> Note on ⬜ in **Impl** column: the SDK method does not yet exist in `lab-apis`. Note on ⬜ in MCP/CLI/API for SDK-implemented rows: the action is implemented in the SDK but not yet exposed as a dispatch action.

---

## System

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/system/status` | `system_status()` | `system.status` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/health` | `health_checks()` | `system.health` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/diskspace` | `disk_space()` | `system.disk-space` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/update` | `updates()` | `system.updates` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file` | `log_files()` | `system.logs` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/{filename}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/update` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/update/{filename}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/restart` | `system_restart()` | `system.restart` (**destructive**) | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/shutdown` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/backup` | `system_backup()` | `system.backup` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/backup/restore/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/backup/restore/upload` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/system/backup/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/task` | `system_tasks()` | `system.task` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/task/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/routes` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/routes/duplicate` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Movie

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/movie` | `movie_list()` | `movie.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/{id}` | `movie_get(id)` | `movie.get` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup` | `movie_lookup(term)` | `movie.lookup` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup/tmdb` | — (covered by `movie.lookup`) | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup/imdb` | — (covered by `movie.lookup`) | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/movie` | `movie_add(movie)` | `movie.add` | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/movie/{id}` | `movie_edit(id, body)` | `movie.edit` | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/movie/{id}` | `movie_delete(id, delete_files)` | `movie.delete` (**destructive**) | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/movie/editor` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/movie/editor` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/movie/import` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/{id}/folder` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `movie.add` params: `tmdb_id` (i64, required), `title` (string, required), `quality_profile_id` (i64, required), `root_folder_path` (string, required), `monitored` (bool, optional, default true), `year` (i32, optional, default 0).
> `movie.delete` params: `id` (i64, required), `delete_files` (bool, optional, default false).

---

## Movie File

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/moviefile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/moviefile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/moviefile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/moviefile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/moviefile/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/moviefile/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/moviefile/editor` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Queue

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/queue` | `queue_list()` | `queue.list` | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/queue/{id}` | `queue_remove(id, remove_from_client, blocklist)` | `queue.remove` (**destructive**) | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/queue/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/queue/details` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/queue/status` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/queue/grab/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/queue/grab/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `queue.remove` params: `id` (i64, required), `remove_from_client` (bool, optional, default true), `blocklist` (bool, optional, default false).

---

## Calendar

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/calendar` | `calendar_list(start, end)` | `calendar.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/feed/v3/calendar/radarr.ics` | — | — | ⬜ | ⬜ | — | — |

> `calendar.list` params: `start` (string ISO 8601, optional, default today), `end` (string ISO 8601, optional, default 7 days from now).

---

## Command

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| POST | `/api/v3/command` (refresh movie) | `command_refresh_movie(movie_id)` | `command.refresh` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/command` (movies search) | `command_movies_search(ids)` | `command.search` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/command` (task execute) | `system_task_execute(name)` | `system.task-execute` (**destructive**) | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/command/{id}` | `command_get(id)` | `command.get` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/command` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/command/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `command.refresh` params: `movie_id` (i64, optional — omit to refresh all).
> `command.search` params: `movie_ids` (i64[], required).
> `command.get` params: `id` (i64, required).

---

## History

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/history` | `history_list(page, page_size)` | `history.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/history/movie` | `history_movie(movie_id)` | `history.movie` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/history/since` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/history/failed/{id}` | `history_failed_retry(id)` | `history.failed-retry` | ✅ | ⬜ | ⬜ | ⬜ |

> `history.list` params: `page` (u32, optional, default 1), `page_size` (u32, optional, default 10).

---

## Blocklist

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/blocklist` | `blocklist_list()` | `blocklist.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/blocklist/movie` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/blocklist/{id}` | `blocklist_delete(id)` | `blocklist.delete` (**destructive**) | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/blocklist/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> Note: `blocklist.list` dispatch logic lives in `dispatch/radarr/history.rs` (co-located with history dispatch).

---

## Release

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/release` | `release_search(movie_id)` | `release.search` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/release` | `release_grab(release)` | `release.grab` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/release/push` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `release.search` params: `movie_id` (i64, required).

---

## Release Profile

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | `/api/v3/releaseprofile` | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/releaseprofile` | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/releaseprofile/{id}` | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/releaseprofile/{id}` | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/releaseprofile/{id}` | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Indexer

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/indexer` | `indexer_list()` | `indexer.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/test` | `indexer_test(id)` | `indexer.test` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/indexer/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/indexer/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/indexer/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/indexer/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/action/{name}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/indexer/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/indexer/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/testall` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `indexer.test` params: `id` (i64, required).

---

## Quality Profile

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/qualityprofile` | `quality_profile_list()` | `quality-profile.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/qualityprofile` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualityprofile/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualityprofile/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/qualityprofile/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualityprofile/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Quality Definition

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/qualitydefinition` | `quality_definition_list()` | `quality-definition.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualitydefinition/limits` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualitydefinition/update` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualitydefinition/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualitydefinition/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Root Folder

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/rootfolder` | `root_folder_list()` | `root-folder.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/rootfolder` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/rootfolder/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/rootfolder/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Tag

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/tag` | `tag_list()` | `tag.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/detail` | `tag_detail_list()` | `tag.detail-list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/tag` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/tag/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/tag/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/detail/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Download Client

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/downloadclient` | `download_client_list()` | `download-client.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/test` | `download_client_get(id)` + `download_client_test(dc)` | `download-client.test` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/downloadclient/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/downloadclient/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/downloadclient/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/downloadclient/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/action/{name}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/downloadclient/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/downloadclient/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/testall` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `download-client.test` params: `id` (i64, required). The dispatch fetches the client by ID then calls `download_client_test()` on it.

---

## Remote Path Mapping

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/remotepathmapping` | `remote_path_mapping_list()` | `remote-path-mapping.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/remotepathmapping` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/remotepathmapping/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/remotepathmapping/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/remotepathmapping/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Config

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/config/host` | `host_config_get()` | `config.host` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming` | `naming_config_get()` | `config.naming` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/ui` | `ui_config_get()` | `config.ui` | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/host/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/host/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/naming/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming/examples` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/ui/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/ui/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/downloadclient` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/downloadclient/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/downloadclient/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/importlist` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/importlist/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/importlist/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/indexer` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/indexer/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/indexer/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/mediamanagement` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/mediamanagement/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/mediamanagement/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/metadata` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/metadata/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/metadata/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Notification

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/notification` | `notification_list()` | `notification.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/test` | `notification_test(id)` | `notification.test` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/notification/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/notification/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/notification/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/notification/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/action/{name}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/testall` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `notification.test` params: `id` (i64, required).

---

## Import List

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/importlist` | `import_list_list()` | `import-list.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/movie` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/movie` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/importlist/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/importlist/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/action/{name}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/importlist/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/importlist/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/test` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/testall` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Exclusions (Import List)

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/exclusions` | `import_list_exclusion_list()` | `import-list.exclusion-list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/exclusions/paged` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/exclusions` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/exclusions/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/exclusions/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/exclusions/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/exclusions/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/exclusions/bulk` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Language

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/language` | `language_list()` | `language.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/language/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Metadata

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/metadata` | `metadata_list()` | `metadata.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/metadata/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/metadata/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/metadata/{id}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/metadata/schema` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/action/{name}` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/test` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/testall` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Filesystem & Manual Import

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|-----------|-----------------|------|-----|-----|-----|
| GET | `/api/v3/filesystem` | `filesystem_list(path)` | `filesystem.list` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/filesystem/mediafiles` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/filesystem/type` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/manualimport` | `manual_import_list(path)` | — (not yet a dispatch action) | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/manualimport` | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

> `filesystem.list` params: `path` (string, required).
> `manual_import_list()` exists in the SDK (Impl ✅) but has **no dispatch action** — it is not exposed through MCP, CLI, or API yet.

---

## Rename / Parse / Wanted

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/rename` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/parse` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/wanted/cutoff` | `wanted_cutoff(page, page_size)` — `wanted.cutoff` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/wanted/missing` | `wanted_missing(page, page_size)` — `wanted.missing` | ✅ | ⬜ | ⬜ | ⬜ |

---

## Alternative Titles

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/alttitle` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/alttitle/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Auto Tagging

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/autotagging` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/autotagging` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/autotagging/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/autotagging/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/autotagging/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/autotagging/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Collection

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/collection` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/collection` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/collection/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/collection/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Credit

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/credit` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/credit/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Custom Filter

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/customfilter` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/customfilter` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/customfilter/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/customfilter/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/customfilter/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Custom Format

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/customformat` | `customformat_list()` — `customformat.list` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/customformat` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/customformat/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/customformat/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/customformat/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/customformat/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/customformat/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/customformat/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Delay Profile

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/delayprofile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/delayprofile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/delayprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/delayprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/delayprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/delayprofile/reorder/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Misc

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/extrafile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/indexerflag` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/localization` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/localization/language` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/mediacover/{movieId}/{filename}` | — | ⬜ | — | — | ⬜ |
| GET | `/ping` | — (health check) | ⬜ | — | — | ⬜ |
| POST | `/login` | `login(req)` | ✅ | — | — | ⬜ |
| GET | `/logout` | `logout()` | ✅ | — | — | ⬜ |

---

## Coverage Summary

| Category | Total Endpoints | SDK Impl | Dispatch Actions | MCP Wired | CLI Wired | API Wired |
|----------|----------------|----------|------------------|-----------|-----------|-----------|
| System | 19 | 5 | 5 (`system.*`) | 5 | 5 | 5 |
| Movie | 12 | 5 | 5 (`movie.*`) | 5 | 5 | 5 |
| Movie File | 7 | 0 | 0 | 0 | 0 | 0 |
| Queue | 7 | 2 | 2 (`queue.list`, `queue.remove`) | 2 | 2 | 2 |
| Calendar | 2 | 1 | 1 (`calendar.list`) | 1 | 1 | 1 |
| Command | 5 | 3 | 3 (`command.*`) | 3 | 3 | 3 |
| History | 4 | 1 | 1 (`history.list`) | 1 | 1 | 1 |
| Blocklist | 4 | 1 | 1 (`blocklist.list`) | 1 | 1 | 1 |
| Release | 3 | 1 | 1 (`release.search`) | 1 | 1 | 1 |
| Release Profile | 5 | 0 | 0 | 0 | 0 | 0 |
| Indexer | 11 | 2 | 2 (`indexer.list`, `indexer.test`) | 2 | 2 | 2 |
| Quality Profile | 6 | 1 | 1 (`quality-profile.list`) | 1 | 1 | 1 |
| Quality Definition | 5 | 1 | 1 (`quality-definition.list`) | 1 | 1 | 1 |
| Root Folder | 4 | 1 | 1 (`root-folder.list`) | 1 | 1 | 1 |
| Tag | 7 | 2 | 2 (`tag.list`, `tag.detail-list`) | 2 | 2 | 2 |
| Download Client | 11 | 2 | 2 (`download-client.list`, `download-client.test`) | 2 | 2 | 2 |
| Remote Path Mapping | 5 | 1 | 1 (`remote-path-mapping.list`) | 1 | 1 | 1 |
| Config | 25 | 3 | 3 (`config.host`, `config.naming`, `config.ui`) | 3 | 3 | 3 |
| Notification | 9 | 2 | 2 (`notification.list`, `notification.test`) | 2 | 2 | 2 |
| Import List | 13 | 1 | 1 (`import-list.list`) | 1 | 1 | 1 |
| Exclusions | 8 | 1 | 1 (`import-list.exclusion-list`) | 1 | 1 | 1 |
| Language | 2 | 1 | 1 (`language.list`) | 1 | 1 | 1 |
| Metadata | 9 | 1 | 1 (`metadata.list`) | 1 | 1 | 1 |
| Filesystem / Manual Import | 5 | 2 | 1 (`filesystem.list`) | 1 | 1 | 1 |
| Rename / Parse / Wanted | 4 | 0 | 0 | 0 | 0 | 0 |
| Alt Titles | 2 | 0 | 0 | 0 | 0 | 0 |
| Auto Tagging | 6 | 0 | 0 | 0 | 0 | 0 |
| Collection | 4 | 0 | 0 | 0 | 0 | 0 |
| Credit | 2 | 0 | 0 | 0 | 0 | 0 |
| Custom Filter | 5 | 0 | 0 | 0 | 0 | 0 |
| Custom Format | 8 | 0 | 0 | 0 | 0 | 0 |
| Delay Profile | 6 | 0 | 0 | 0 | 0 | 0 |
| Misc | 8 | 2 | 0 | 0 | 0 | 0 |
| **Total** | **233** | **42** | **39** | **39** | **39** | **39** |

> **Dispatch Actions** = action strings registered in `dispatch/radarr/catalog.rs` (via sub-modules). All 39 are wired through MCP, CLI, and API.
> **Wired** = dispatch handler confirmed present in source. Does NOT mean live-tested against a running Radarr instance.
> Live testing required to earn ✅ in MCP / CLI / API columns in the detail tables above.
