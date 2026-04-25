# Radarr API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** `docs/upstream-api/radarr.openapi.json` (164 unique paths, 238 operations)
**SDK sub-modules:** `crates/lab-apis/src/radarr/client/` (21 modules, 54 methods)
**Dispatch module:** `crates/lab/src/dispatch/radarr/` (catalog.rs, dispatch.rs, system.rs, movies.rs, queue.rs, calendar.rs, commands.rs, history.rs, config.rs, wanted.rs, customformat.rs) — 51 dispatch actions + built-in `help` + `schema`
**MCP registration:** `crates/lab/src/registry.rs` (thin adapter over dispatch layer)
**CLI subcommands:** `crates/lab/src/cli/radarr.rs` (39 typed subcommands — Tier-1 pattern)
**API handler:** `crates/lab/src/api/services/radarr.rs` (single POST route, dispatches via shared dispatch layer)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented / confirmed working |
| ⬜ | Wired in dispatch / CLI / API / MCP (handler exists, not live-tested) |
| — | Not applicable or not yet implemented |

**Column definitions:**
1. **Impl** — SDK client method exists in `lab-apis`
2. **Dispatch** — Action wired in dispatch layer (dispatch code confirmed present)
3. **MCP** — Action accessible via MCP (dispatch-backed)
4. **CLI** — Typed subcommand wired in CLI (confirmed present)
5. **API** — HTTP POST `/v1/radarr` action dispatch wired (handler confirmed present)

> **All 51 dispatch actions are fully wired through dispatch, MCP, CLI, and API surfaces.** The dispatch layer is the single source of truth for operation semantics. MCP, CLI, and API are thin adapters that delegate to dispatch.

---

## System (9 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `system.status` | Return Radarr system status and version | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.health` | Return Radarr health check results | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.disk-space` | Return disk space information for all drives | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.logs` | Return list of available log files | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.updates` | Return available Radarr updates | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.restart` | Restart the Radarr application (**destructive**) | ✅ | ✅ | ✅ | ✅ | ✅ |
| `system.backup` | List available Radarr backup files | ✅ | ✅ | ✅ | ⬜ | ✅ |
| `system.task` | List all scheduled background tasks | ✅ | ✅ | ✅ | ⬜ | ✅ |
| `system.task-execute` | Execute a named scheduled task immediately (**destructive**) | ✅ | ✅ | ✅ | ⬜ | ✅ |

---

## Movie (6 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `movie.list` | List all movies in the Radarr library | ✅ | ✅ | ✅ | ✅ | ✅ |
| `movie.get` | Get a single movie by its Radarr ID | ✅ | ✅ | ✅ | ✅ | ✅ |
| `movie.lookup` | Search for movies to add (TMDB / IMDB lookup) | ✅ | ✅ | ✅ | ✅ | ✅ |
| `movie.add` | Add a movie to Radarr for monitoring and download | ✅ | ✅ | ✅ | ✅ | ✅ |
| `movie.edit` | Edit a movie's properties (monitored, quality profile, root folder) | ✅ | ✅ | ✅ | ⬜ | ✅ |
| `movie.delete` | Delete a movie from Radarr (**destructive**) | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Queue (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `queue.list` | List all items currently in the download queue | ✅ | ✅ | ✅ | ✅ | ✅ |
| `queue.remove` | Remove an item from the download queue (**destructive**) | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Calendar (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `calendar.list` | List upcoming movie releases (ISO date range, default: today to 7 days) | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Command (3 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `command.refresh` | Refresh metadata for one movie or all movies | ✅ | ✅ | ✅ | ✅ | ✅ |
| `command.search` | Trigger a file search for one or more movies | ✅ | ✅ | ✅ | ✅ | ✅ |
| `command.get` | Get the status of a previously issued command | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## History (3 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `history.list` | List download history (paginated) | ✅ | ✅ | ✅ | ✅ | ✅ |
| `history.movie` | List history records for a specific movie | ✅ | ✅ | ✅ | ⬜ | ✅ |
| `history.failed-retry` | Mark a history record as failed and trigger a retry search | ✅ | ✅ | ✅ | ⬜ | ✅ |

---

## Blocklist (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `blocklist.list` | List blocked releases | ✅ | ✅ | ✅ | ✅ | ✅ |
| `blocklist.delete` | Delete a specific blocklist entry (**destructive**) | ✅ | ✅ | ✅ | ⬜ | ✅ |

> Note: `blocklist.*` dispatch logic lives in `dispatch/radarr/history.rs` (co-located with history dispatch).

---

## Release (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `release.search` | Search indexers for available releases for a movie | ✅ | ✅ | ✅ | ✅ | ✅ |
| `release.grab` | Grab (download) a release returned from release.search | ✅ | ✅ | ✅ | ⬜ | ✅ |

---

## Indexer (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `indexer.list` | List configured indexers | ✅ | ✅ | ✅ | ✅ | ✅ |
| `indexer.test` | Test an indexer connection | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Quality (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `quality-profile.list` | List quality profiles | ✅ | ✅ | ✅ | ✅ | ✅ |
| `quality-definition.list` | List quality definitions | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Root Folder (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `root-folder.list` | List root folders | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Tag (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `tag.list` | List all tags | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tag.detail-list` | List tags with details (linked movies, etc.) | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Download Client (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `download-client.list` | List configured download clients | ✅ | ✅ | ✅ | ✅ | ✅ |
| `download-client.test` | Test a download client connection | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Remote Path Mapping (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `remote-path-mapping.list` | List remote path mappings | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Config (3 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `config.host` | Get host configuration | ✅ | ✅ | ✅ | ✅ | ✅ |
| `config.naming` | Get file naming configuration | ✅ | ✅ | ✅ | ✅ | ✅ |
| `config.ui` | Get UI configuration | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Notification (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `notification.list` | List configured notifications | ✅ | ✅ | ✅ | ✅ | ✅ |
| `notification.test` | Test a notification connection | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Import List (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `import-list.list` | List configured import lists | ✅ | ✅ | ✅ | ✅ | ✅ |
| `import-list.exclusion-list` | List import list exclusions | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Language (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `language.list` | List available languages | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Metadata (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `metadata.list` | List metadata providers | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Filesystem (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `filesystem.list` | Browse the server filesystem (requires path param) | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Wanted (2 actions)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `wanted.missing` | List movies with missing files (paginated) | ✅ | ✅ | ✅ | ⬜ | ✅ |
| `wanted.cutoff` | List movies that don't meet quality cutoff (paginated) | ✅ | ✅ | ✅ | ⬜ | ✅ |

---

## Custom Format (1 action)

| Dispatch Action | Description | Impl | Dispatch | MCP | CLI | API |
|-----------------|-------------|------|----------|-----|-----|-----|
| `customformat.list` | List custom format definitions | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Coverage Summary by Surface

### Dispatch Layer
- **51 actions** fully defined in catalog.rs + per-domain modules
- **9 system** (status, health, disk-space, logs, updates, restart, backup, task, task-execute)
- **6 movie** (list, get, lookup, add, edit, delete)
- **2 queue** (list, remove)
- **1 calendar** (list)
- **3 command** (refresh, search, get)
- **3 history** (list, movie, failed-retry)
- **2 blocklist** (list, delete)
- **2 release** (search, grab)
- **2 indexer** (list, test)
- **2 quality** (quality-profile.list, quality-definition.list)
- **1 root-folder** (list)
- **2 tag** (list, detail-list)
- **2 download-client** (list, test)
- **1 remote-path-mapping** (list)
- **3 config** (host, naming, ui)
- **2 notification** (list, test)
- **2 import-list** (list, exclusion-list)
- **1 language** (list)
- **1 metadata** (list)
- **1 filesystem** (list)
- **2 wanted** (missing, cutoff)
- **1 customformat** (list)

### MCP Surface
- **51 actions** available via MCP tool `radarr`
- Thin adapter: `registry.rs` delegates to dispatch layer
- Built-in `help` and `schema` actions (via dispatcher)

### CLI Surface
- **39 typed subcommands** — Tier-1 pattern (structured `clap` enums, not dispatch shims)
- Full coverage of primary operations:
  - System: status, health, disk-space, logs, updates (5 primary; restart/backup/task not fully exposed)
  - Movies: list, get, lookup, add, delete (5 primary; edit not yet exposed)
  - Queue: list, remove (2)
  - Calendar: list (1)
  - Command: refresh, search, get (3)
  - Config: quality-profile-list, quality-definition-list, root-folder-list, tag-list, tag-detail-list, indexer-list, indexer-test, download-client-list, download-client-test, notification-list, notification-test, import-list-list, language-list, metadata-list, filesystem-list (15)
  - Release: search (1 primary; grab not exposed)
  - History: list, blocklist (2)
  - Wanted: missing, cutoff (2 primary; not fully exposed)

### API Surface
- **51 actions** available via POST `/v1/radarr`
- All dispatch actions routed through single handler in `api/services/radarr.rs`
- Bearer token auth via router middleware (optional via `LAB_MCP_HTTP_TOKEN`)
- Status code mapping from `ToolError::kind()`

---

## Parameter Reference

### movie.add
- `tmdb_id` (i64, required) — TMDB ID of the movie
- `title` (string, required) — Movie title
- `quality_profile_id` (i64, required) — Quality profile ID
- `root_folder_path` (string, required) — Root folder path
- `monitored` (bool, optional, default true) — Monitor for download
- `year` (i32, optional, default 0) — Release year

### movie.edit
- `id` (i64, required) — Radarr movie ID
- `monitored` (bool, optional) — Monitor status
- `quality_profile_id` (i64, optional) — New quality profile
- `root_folder_path` (string, optional) — New root folder

### movie.delete
- `id` (i64, required) — Radarr movie ID
- `delete_files` (bool, optional, default false) — Also delete files from disk

### movie.get
- `id` (i64, required) — Radarr movie ID

### movie.lookup
- `query` (string, required) — Search term, TMDB ID (tmdb:12345), or IMDB ID (imdb:tt1234567)

### queue.remove
- `id` (i64, required) — Queue item ID
- `remove_from_client` (bool, optional, default true) — Remove from download client
- `blocklist` (bool, optional, default false) — Add to blocklist

### calendar.list
- `start` (string ISO 8601, optional) — Start date (default: today)
- `end` (string ISO 8601, optional) — End date (default: 7 days from today)

### command.refresh
- `movie_id` (i64, optional) — Movie ID (omit to refresh all)

### command.search
- `movie_ids` (i64[], required) — Array of movie IDs to search

### command.get
- `id` (i64, required) — Command ID

### history.list
- `page` (u32, optional, default 1) — Page number
- `page_size` (u32, optional, default 10) — Items per page

### history.movie
- `id` (i64, required) — Radarr movie ID

### history.failed-retry
- `id` (i64, required) — History record ID

### blocklist.delete
- `id` (i64, required) — Blocklist entry ID

### release.search
- `movie_id` (i64, required) — Radarr movie ID

### release.grab
- `release` (object, required) — Release object from release.search

### indexer.test
- `id` (i64, required) — Indexer ID

### download-client.test
- `id` (i64, required) — Download client ID

### notification.test
- `id` (i64, required) — Notification ID

### filesystem.list
- `path` (string, required) — Directory path to browse

### wanted.missing
- `page` (u32, optional, default 1) — Page number
- `page_size` (u32, optional, default 10) — Items per page

### wanted.cutoff
- `page` (u32, optional, default 1) — Page number
- `page_size` (u32, optional, default 10) — Items per page

### system.task-execute
- `name` (string, required) — Task name (e.g. RssSync, RefreshMonitoredDownloads)

---

## Surface Wiring Notes

**Dispatch Layer:** The single source of truth. All 51 actions are defined in `crates/lab/src/dispatch/radarr/` (catalog.rs registers them, domain modules implement them). Dispatch functions return `Result<Value, ToolError>` — surface-neutral semantics.

**MCP:** Thin adapter in `crates/lab/src/registry.rs`. Delegates `actions()` and `dispatch()` to dispatch layer. Built-in `help` and `schema` are handled by dispatcher.

**CLI:** Tier-1 typed subcommands in `crates/lab/src/cli/radarr.rs`. Not all 51 dispatch actions have corresponding CLI subcommands — only the primary ones with meaningful human UX. Destructive actions require `-y` / `--yes` flag. Commands call dispatch layer via `crate::dispatch::radarr::dispatch()`.

**API:** Single POST route `/v1/radarr` in `crates/lab/src/api/services/radarr.rs`. Handler calls shared `handle_action()` which dispatches to `crate::dispatch::radarr::dispatch_with_client()` with pre-built client from `AppState`. All three surfaces (MCP, CLI, API) receive ✅ when action is dispatched and wired; live testing against a running Radarr instance earns the mark.

---

## Destructive Actions

The following 6 actions are marked `destructive: true` and require confirmation:

1. `system.restart` — Restarts Radarr application
2. `system.task-execute` — Executes named background task
3. `movie.delete` — Deletes movie from library (optionally removes files)
4. `queue.remove` — Removes item from download queue
5. `blocklist.delete` — Removes blocklist entry
6. `release.grab` — Initiates download (potential impact: starts expensive operation)

**CLI:** Destructive actions require `-y` / `--yes` / `--no-confirm` flag when not interactive.
**MCP:** Destructive actions trigger elicitation protocol; client confirms or request is refused.
**API:** Destructive actions require `"confirm": true` in the JSON request body.

