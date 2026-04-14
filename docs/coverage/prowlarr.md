# Prowlarr API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** docs/api-specs/prowlarr.openapi.json
**OpenAPI version:** 3.0.4
**API version:** 1.0.0
**Paths:** 129
**Servers:** {protocol}://{hostpath}
**Security schemes:** X-Api-Key, apikey

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented (SDK + dispatch + MCP/CLI/API) |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Implementation Notes

- CLI is a thin dispatch shim: `lab prowlarr <action> [--params <json>]`. All dispatch actions are available via CLI.
- MCP: one tool `prowlarr` with action dispatch.
- API: `POST /v1/prowlarr`.
- Auth: `PROWLARR_URL` + `PROWLARR_API_KEY` env vars; key sent as `X-Api-Key` header.
- There is no `health` dispatch action. The SDK has a `probe()` method used internally by `lab doctor`.

## Implemented Action Catalog

| Action | SDK Method | Destructive | Params |
|--------|-----------|-------------|--------|
| `help` | built-in | no | — |
| `schema` | built-in | no | `action: string` |
| `indexers.list` | `indexers_list()` | no | — |
| `indexers.get` | `indexer_get(id)` | no | `id: integer` |
| `indexers.delete` | `indexer_delete(id)` | **yes** | `id: integer` |
| `indexers.test` | `indexer_test(id)` | no | `id: integer` |
| `indexers.testall` | `indexers_testall()` | no | — |
| `indexers.categories` | `indexer_categories()` | no | — |
| `history.list` | `history_list(query)` | no | `page?: integer`, `page_size?: integer`, `sort_key?: string`, `sort_dir?: string`, `indexer_id?: integer` |
| `applications.list` | `applications_list()` | no | — |
| `applications.get` | `application_get(id)` | no | `id: integer` |
| `applications.delete` | `application_delete(id)` | **yes** | `id: integer` |
| `indexer.edit` | `indexer_edit(id, body)` | no | `id: integer`, `body: object` |
| `indexer.add` | `indexer_add(body)` | no | `body: object` |
| `indexer.stats` | `indexer_stats()` | no | — |
| `indexer.status` | `indexer_status()` | no | — |
| `indexer.search` | `indexer_search(query, indexer_ids, categories, type)` | no | `query: string`, `indexer_ids?: array[integer]`, `categories?: array[integer]`, `search_type?: string` |
| `indexer.grab` | `indexer_grab(guid)` | no | `guid: string` |
| `history.indexer` | `history_indexer(id)` | no | `id: integer` |
| `application.add` | `application_add(body)` | no | `body: object` |
| `system.restart` | `system_restart()` | **yes** | — |
| `system.backup` | `system_backup()` | no | — |
| `tag.list` | `tag_list()` | no | — |
| `system.status` | `system_status()` | no | — |
| `system.health` | `system_health()` | no | — |

## Live Test Evidence

Live smoke tests run 2026-04-12 against `https://prowlarr.tootie.tv` (v2.3.5.5327).

| Surface | Command | Result |
|---------|---------|--------|
| CLI | `lab prowlarr system.status` | v2.3.5.5327, isDocker=true |
| CLI | `lab prowlarr indexers.list` | 3 indexers: NZBgeek, NzbPlanet, TorrentLeech |
| CLI | `lab prowlarr system.health` | health warnings returned |
| MCP | `mcporter call lab.prowlarr action=system.status` | `ok=true`, version=2.3.5.5327 |
| MCP | `mcporter call lab.prowlarr action=indexers.list` | `ok=true`, 3 indexers |
| API | `POST /v1/prowlarr {"action":"system.status"}` | version=2.3.5.5327, appName=Prowlarr |
| API | `POST /v1/prowlarr {"action":"indexers.list"}` | 3 indexers listed |

## Endpoint Inventory

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | / | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/applications | `applications_list` → `applications.list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/applications | `application_add` → `application.add` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/applications/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/applications/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/applications/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/applications/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/applications/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/applications/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/applications/{id} | `application_delete` → `applications.delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/applications/{id} | `application_get` → `applications.get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v1/applications/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/appprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/appprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/appprofile/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/appprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/appprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/appprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/command | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/command | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/command/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/command/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/development | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/development/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/config/development/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/config/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/host | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/host/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/config/host/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/ui | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/config/ui/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/config/ui/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/customfilter | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/customfilter | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/customfilter/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/downloadclient | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/downloadclient/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/downloadclient/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/downloadclient/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/downloadclient/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/downloadclient/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/downloadclient/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/downloadclient/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/filesystem | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/filesystem/type | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/health | `system_health` → `system.health` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/history | `history_list` → `history.list` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/history/indexer | `history_indexer` → `history.indexer` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/history/since | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexer | `indexers_list` → `indexers.list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/indexer | `indexer_add` → `indexer.add` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/indexer/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/indexer/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/indexer/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexer/categories | `indexer_categories` → `indexers.categories` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/indexer/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/indexer/test | `indexer_test` (GET+POST) → `indexers.test` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/indexer/testall | `indexers_testall` → `indexers.testall` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /api/v1/indexer/{id} | `indexer_delete` → `indexers.delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/indexer/{id} | `indexer_get` → `indexers.get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /api/v1/indexer/{id} | `indexer_edit` → `indexer.edit` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/indexer/{id}/download | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexer/{id}/newznab | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexerproxy | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/indexerproxy | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/indexerproxy/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexerproxy/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/indexerproxy/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/indexerproxy/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/indexerproxy/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexerproxy/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/indexerproxy/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/indexerstats | `indexer_stats` → `indexer.stats` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/indexerstatus | `indexer_status` → `indexer.status` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/localization | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/localization/options | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/log | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/log/file | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/log/file/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/log/file/update/{filename} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/log/file/{filename} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/notification | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/notification/action/{name} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/notification/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/notification/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/notification/testall | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/notification/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/search | `indexer_search` → `indexer.search` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/search | `indexer_grab` → `indexer.grab` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/search/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/system/backup | `system_backup` → `system.backup` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/system/backup/restore/upload | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/system/backup/restore/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/system/backup/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/system/restart | `system_restart` → `system.restart` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/system/routes | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/system/routes/duplicate | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v1/system/shutdown | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/system/status | `system_status` → `system.status` | ✅ | ✅ | ✅ | ✅ |
| GET | /api/v1/system/task | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/system/task/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/tag | `tag_list` → `tag.list` | ✅ | ✅ | ✅ | ✅ |
| POST | /api/v1/tag | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/tag/detail | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/tag/detail/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v1/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v1/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v1/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /content/{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /logout | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| HEAD | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{id}/api | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{id}/download | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
