# Radarr API Coverage

**Last updated:** 2026-04-08  
**OpenAPI spec:** `docs/upstream-api/radarr.openapi.json` (164 unique paths, 238 operations)  
**SDK sub-modules:** `crates/lab-apis/src/radarr/client/` (19 modules, 42 methods)  
**MCP actions:** `crates/lab/src/mcp/services/radarr.rs` (39 actions + built-in `help`)  
**CLI subcommands:** `crates/lab/src/cli/radarr.rs` (39 subcommands)  
**HTTP API handler:** `crates/lab/src/api/services/radarr.rs` (single dispatch → same as MCP)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented / confirmed working |
| ⬜ | Not yet implemented, or wired but not live-tested |
| — | Not applicable |

**Column definitions:**
1. **Impl** — SDK client method exists in `lab-apis`
2. **MCP** — Action wired in MCP dispatcher (dispatch code exists; ✅ requires live-test confirmation)
3. **CLI** — Subcommand wired in CLI (subcommand exists; ✅ requires live-test confirmation)
4. **API** — HTTP POST `/v1/radarr` action dispatch wired (handler exists; ✅ requires live-test confirmation)

> ⚠️ **Summary counts for MCP / CLI / API reflect wired handlers, not live-tested endpoints.** In the detail rows above, ⬜ means the action is either not yet wired or wired but awaiting live verification. A cell turns ✅ only after manual testing against a running Radarr instance — CI test pass alone does not qualify.

---

## System

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/system/status` | `system_status()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/health` | `health_checks()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/diskspace` | `disk_space()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/update` | `updates()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file` | `log_files()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/{filename}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/update` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log/file/update/{filename}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/log` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/restart` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/shutdown` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/backup` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/backup/restore/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/system/backup/restore/upload` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/system/backup/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/task` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/task/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/routes` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/system/routes/duplicate` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Movie

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/movie` | `movie_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/{id}` | `movie_get(id)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup` | `movie_lookup(term)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup/tmdb` | — (use `movie.lookup`) | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/lookup/imdb` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/movie` | `movie_add(movie)` | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/movie/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/movie/{id}` | `movie_delete(id, delete_files)` | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/movie/editor` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/movie/editor` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/movie/import` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/movie/{id}/folder` | — | ⬜ | ⬜ | ⬜ | ⬜ |

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

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/queue` | `queue_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/queue/{id}` | `queue_remove(id, ...)` | ✅ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/queue/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/queue/details` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/queue/status` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/queue/grab/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/queue/grab/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Calendar

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/calendar` | `calendar_list(start, end)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/feed/v3/calendar/radarr.ics` | — | ⬜ | ⬜ | — | — |

---

## Command

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| POST | `/api/v3/command` (refresh movie) | `command_refresh_movie(movie_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/command` (movies search) | `command_movies_search(ids)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/command/{id}` | `command_get(id)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/command` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/command/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## History

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/history` | `history_list(page, page_size)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/history/movie` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/history/since` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/history/failed/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Blocklist

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/blocklist` | `blocklist_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/blocklist/movie` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/blocklist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/blocklist/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Release

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/release` | `release_search(movie_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/release` | — (grab/download release) | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/release/push` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Release Profile

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/releaseprofile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/releaseprofile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/releaseprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/releaseprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/releaseprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Indexer

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/indexer` | `indexer_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/test` | `indexer_test(id)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/indexer/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/indexer/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/indexer/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/indexer/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/action/{name}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/indexer/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/indexer/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/indexer/testall` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Quality Profile

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/qualityprofile` | `quality_profile_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/qualityprofile` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualityprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualityprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/qualityprofile/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualityprofile/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Quality Definition

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/qualitydefinition` | `quality_definition_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualitydefinition/limits` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualitydefinition/update` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/qualitydefinition/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/qualitydefinition/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Root Folder

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/rootfolder` | `root_folder_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/rootfolder` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/rootfolder/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/rootfolder/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Tag

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/tag` | `tag_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/detail` | `tag_detail_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/tag` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/tag/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/tag/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/tag/detail/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Download Client

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/downloadclient` | `download_client_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/test` | `download_client_test(id)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/downloadclient/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/downloadclient/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/downloadclient/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/downloadclient/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/action/{name}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/downloadclient/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/downloadclient/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/downloadclient/testall` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Remote Path Mapping

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/remotepathmapping` | `remote_path_mapping_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/remotepathmapping` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/remotepathmapping/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/remotepathmapping/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/remotepathmapping/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Config

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/config/host` | `host_config_get()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming` | `naming_config_get()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/ui` | `ui_config_get()` | ✅ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/host/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/host/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/naming/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/naming/examples` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/ui/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/ui/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/downloadclient` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/downloadclient/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/downloadclient/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/importlist` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/importlist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/importlist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/indexer` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/indexer/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/indexer/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/mediamanagement` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/mediamanagement/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/mediamanagement/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/metadata` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/config/metadata/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/config/metadata/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Notification

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/notification` | `notification_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/test` | `notification_test(id)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/notification/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/notification/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/notification/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/notification/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/action/{name}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/notification/testall` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Import List

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/importlist` | `import_list_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/movie` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/movie` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/importlist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/importlist/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/importlist/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/action/{name}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/importlist/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/importlist/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/test` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/importlist/testall` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Exclusions (Import List)

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/exclusions` | `import_list_exclusion_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/exclusions/paged` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/exclusions` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/exclusions/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/exclusions/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/exclusions/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/exclusions/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/exclusions/bulk` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Language

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/language` | `language_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/language/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Metadata

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/metadata` | `metadata_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/metadata/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | `/api/v3/metadata/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | `/api/v3/metadata/{id}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/metadata/schema` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/action/{name}` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/test` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/metadata/testall` | — | ⬜ | ⬜ | ⬜ | ⬜ |

---

## Filesystem & Manual Import

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/filesystem` | `filesystem_list(path)` | ✅ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/filesystem/mediafiles` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/filesystem/type` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/manualimport` | `manual_import_list(path)` | ✅ | ⬜ | ⬜ | ⬜ |
| POST | `/api/v3/manualimport` | — | ⬜ | ⬜ | ⬜ | ⬜ |

> **Note:** `filesystem_list()` is wired in MCP and CLI as `filesystem.list`. `manual_import_list()` exists in the SDK (Impl ✅) but is **not yet wired** in MCP, CLI, or API — the MCP/CLI/API cells for that row remain ⬜ until a dedicated action is added.

---

## Rename / Parse / Wanted

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | `/api/v3/rename` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/parse` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/wanted/cutoff` | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | `/api/v3/wanted/missing` | — | ⬜ | ⬜ | ⬜ | ⬜ |

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
| GET | `/api/v3/customformat` | — | ⬜ | ⬜ | ⬜ | ⬜ |
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

| Category | Total Endpoints | SDK Impl | MCP Wired | CLI Wired | API Wired |
|----------|----------------|----------|-----------|-----------|-----------|
| System | 19 | 5 | 5 | 5 | 5 |
| Movie | 12 | 5 | 5 | 5 | 5 |
| Movie File | 7 | 0 | 0 | 0 | 0 |
| Queue | 7 | 2 | 2 | 2 | 2 |
| Calendar | 2 | 1 | 1 | 1 | 1 |
| Command | 5 | 3 | 3 | 3 | 3 |
| History | 4 | 1 | 1 | 1 | 1 |
| Blocklist | 4 | 1 | 1 | 1 | 1 |
| Release | 3 | 1 | 1 | 1 | 1 |
| Release Profile | 5 | 0 | 0 | 0 | 0 |
| Indexer | 11 | 2 | 2 | 2 | 2 |
| Quality Profile | 6 | 1 | 1 | 1 | 1 |
| Quality Definition | 5 | 1 | 1 | 1 | 1 |
| Root Folder | 4 | 1 | 1 | 1 | 1 |
| Tag | 7 | 2 | 2 | 2 | 2 |
| Download Client | 11 | 2 | 2 | 2 | 2 |
| Remote Path Mapping | 5 | 1 | 1 | 1 | 1 |
| Config | 25 | 3 | 3 | 3 | 3 |
| Notification | 9 | 2 | 2 | 2 | 2 |
| Import List | 13 | 1 | 1 | 1 | 1 |
| Exclusions | 8 | 1 | 1 | 1 | 1 |
| Language | 2 | 1 | 1 | 1 | 1 |
| Metadata | 9 | 1 | 1 | 1 | 1 |
| Filesystem / Manual Import | 5 | 2 | 1 | 1 | 1 |
| Rename / Parse / Wanted | 4 | 0 | 0 | 0 | 0 |
| Alt Titles | 2 | 0 | 0 | 0 | 0 |
| Auto Tagging | 6 | 0 | 0 | 0 | 0 |
| Collection | 4 | 0 | 0 | 0 | 0 |
| Credit | 2 | 0 | 0 | 0 | 0 |
| Custom Filter | 5 | 0 | 0 | 0 | 0 |
| Custom Format | 8 | 0 | 0 | 0 | 0 |
| Delay Profile | 6 | 0 | 0 | 0 | 0 |
| Misc | 8 | 2 | 0 | 0 | 0 |
| **Total** | **233** | **42** | **40** | **39** | **40** |

> **Wired** = action/subcommand exists in the handler. Does NOT mean live-tested.  
> Live testing required to earn ✅ in MCP / CLI / API columns above.
