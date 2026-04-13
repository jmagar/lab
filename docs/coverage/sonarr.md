# Sonarr API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** docs/upstream-api/sonarr.openapi.json
**OpenAPI version:** 3.0.1
**API version:** 3.0.0
**Paths:** 162
**Servers:** {protocol}://{hostpath}
**Security schemes:** X-Api-Key

## Implementation Status

The Sonarr service is **fully onboarded** as of 2026-04-13. The shared dispatch layer
(`crates/lab/src/dispatch/sonarr/`) is implemented with catalog, client, params, and
dispatch modules. The SDK client (`crates/lab-apis/src/sonarr/`) covers the priority
endpoints listed below.

### Implemented Actions

| Action | SDK Method | Endpoint |
|--------|-----------|---------|
| `series.list` | `series_list()` | GET /api/v3/series |
| `series.get` | `series_get(id)` | GET /api/v3/series/{id} |
| `series.lookup` | `series_lookup(query)` | GET /api/v3/series/lookup |
| `series.add` | `series_add(req)` | POST /api/v3/series |
| `series.delete` | `series_delete(id, delete_files)` | DELETE /api/v3/series/{id} |
| `episode.list` | `episode_list(series_id)` | GET /api/v3/episode |
| `episode.get` | `episode_get(id)` | GET /api/v3/episode/{id} |
| `queue.list` | `queue_list(query)` | GET /api/v3/queue |
| `queue.delete` | `queue_delete(id, blocklist)` | DELETE /api/v3/queue/{id} |
| `history.list` | `history_list(query)` | GET /api/v3/history |
| `wanted.list` | `wanted_missing(page, page_size)` | GET /api/v3/wanted/missing |
| `calendar.list` | `calendar_list(query)` | GET /api/v3/calendar |
| `health` | `health()` | GET /api/v3/health |
| `system.status` | `system_status()` | GET /api/v3/system/status |
| `tag.list` | `tag_list()` | GET /api/v3/tag |
| `tag.create` | `tag_create(label)` | POST /api/v3/tag |
| `tag.delete` | `tag_delete(id)` | DELETE /api/v3/tag/{id} |
| `rootfolder.list` | `rootfolder_list()` | GET /api/v3/rootfolder |
| `qualityprofile.list` | `qualityprofile_list()` | GET /api/v3/qualityprofile |
| `languageprofile.list` | `languageprofile_list()` | GET /api/v3/languageprofile |

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
| GET | /api/v3/blocklist | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/blocklist/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/blocklist/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/calendar | - | ⬜ | ⬜ | ⬜ | ⬜ |
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
| GET | /api/v3/episode | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episode/monitor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episode/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episode/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episodefile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/episodefile/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/episodefile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem/mediafiles | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/filesystem/type | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/health | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/history | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/history/failed/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/history/series | - | ⬜ | ⬜ | ⬜ | ⬜ |
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
| GET | /api/v3/languageprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
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
| GET | /api/v3/qualityprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/qualityprofile | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile/schema | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/qualityprofile/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/queue/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue/details | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/queue/grab/bulk | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/queue/grab/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/queue/status | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/queue/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/release | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/release | - | ⬜ | ⬜ | ⬜ | ⬜ |
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
| GET | /api/v3/rootfolder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/rootfolder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/rootfolder/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/seasonpass | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/series | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/series/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/series/editor | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/series/import | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series/lookup | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/series/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/series/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/series/{id}/folder | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/backup | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/backup/restore/upload | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/backup/restore/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/system/backup/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/restart | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/routes | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/routes/duplicate | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/system/shutdown | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/status | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/task | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/system/task/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api/v3/tag | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag/detail | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag/detail/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api/v3/tag/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/update | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/cutoff | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/cutoff/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/missing | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api/v3/wanted/missing/{id} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /content/{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /feed/v3/calendar/sonarr.ics | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /login | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /logout | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| HEAD | /ping | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{path} | - | ⬜ | ⬜ | ⬜ | ⬜ |
