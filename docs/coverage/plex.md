# Plex API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/api-specs/plex.openapi.yaml
**OpenAPI version:** 3.1.1
**API version:** 1.1.1
**Paths:** 241
**Servers:** https://{IP-description}.{identifier}.plex.direct:{port}, {protocol}://{host}:{port}, {full_server_url}
**Security schemes:** token

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented (SDK + dispatch + MCP/CLI/API) |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Implemented Actions

The following actions are fully implemented across SDK (`lab-apis`), dispatch layer, CLI, MCP, and HTTP API.

- **SDK:** `PlexClient` in `crates/lab-apis/src/plex/client.rs` with individual async methods
- **Dispatch layer:** `crates/lab/src/dispatch/plex/` (catalog, client, params, dispatch)
- **CLI:** thin dispatch shim (`lab plex <action> [--params <json>]`)
- **MCP:** one tool `plex` with action dispatch
- **API:** `POST /v1/plex`
- **Auth:** `PLEX_URL` + `PLEX_TOKEN` env vars; token sent as `X-Plex-Token` header

| Action | SDK Method | Destructive | Params | Returns |
|--------|-----------|-------------|--------|---------|
| `help` | built-in | no | — | Catalog |
| `schema` | built-in | no | `action: string` | Schema |
| `health` | `probe()` | no | — | `{ "status": "ok" }` |
| `server.info` | `server_info()` | no | — | Value (`GET /`) |
| `server.capabilities` | `server_capabilities()` | no | — | Value (`GET /media/providers`) |
| `library.list` | `library_list()` | no | — | Value (`GET /library/sections`) |
| `library.get` | `library_get(section_id)` | no | `section_id: string` | Value |
| `library.scan` | `library_scan(section_id)` | no | `section_id: string` | Value |
| `library.refresh` | `library_refresh(section_id)` | **yes** | `section_id: string` | Value |
| `media.search` | `media_search(params)` | no | `query: string`, `limit?: integer`, `section_id?: string` | Value (`GET /hubs/search`) |
| `media.get` | `media_get(rating_key)` | no | `rating_key: string` | Value (`GET /library/metadata/{id}`) |
| `session.list` | `session_list()` | no | — | Value (`GET /status/sessions`) |
| `session.terminate` | `session_terminate(session_id, reason)` | **yes** | `session_id: string`, `reason?: string` | Value |
| `playlist.list` | `playlist_list()` | no | — | Value (`GET /playlists`) |
| `playlist.get` | `playlist_get(playlist_id)` | no | `playlist_id: string` | Value |
| `playlist.create` | `playlist_create(title, playlist_type, uri)` | **yes** | `title: string`, `playlist_type: string`, `uri?: string` | Value |
| `playlist.delete` | `playlist_delete(playlist_id)` | **yes** | `playlist_id: string` | void |
| `library.browse` | `library_browse(section_id, type_filter, sort)` | no | `section_id: integer`, `type?: string`, `sort?: string` | Value (`GET /library/sections/{id}/all`) |
| `library.empty-trash` | `library_empty_trash(section_id)` | **yes** | `section_id: integer` | void (`PUT /library/sections/{id}/emptyTrash`) |
| `metadata.delete` | `metadata_delete(rating_key)` | **yes** | `rating_key: string` | void (`DELETE /library/metadata/{id}`) |
| `metadata.edit` | `metadata_edit(rating_key, fields)` | no | `rating_key: string`, `fields: object` | Value (`GET /library/metadata/{id}` with query params) |
| `metadata.refresh` | `metadata_refresh(rating_key)` | no | `rating_key: string` | void (`PUT /library/metadata/{id}/refresh`) |
| `session.history` | `session_history(account_id, limit)` | no | `account_id?: integer`, `limit?: integer` | Value (`GET /status/sessions/history/all`) |
| `hubs.continue-watching` | `hubs_continue_watching()` | no | — | Value (`GET /hubs/continueWatching`) |
| `butler.list` | `butler_list()` | no | — | Value (`GET /butler`) |
| `butler.run` | `butler_run(task_name)` | no | `task_name: string` | void (`POST /butler/{task}`) |
| `item.scrobble` | `item_scrobble(rating_key)` | no | `rating_key: string` | void (`GET /:/scrobble`) |
| `item.unscrobble` | `item_unscrobble(rating_key)` | no | `rating_key: string` | void (`GET /:/unscrobble`) |
| `updater.status` | `updater_status()` | no | — | Value (`GET /updater/status`) |

## Surface Coverage

### SDK (`lab-apis/src/plex/client.rs`)

All 29 implemented actions have corresponding async methods in `PlexClient`:

- **Server:** `server_info()`, `server_capabilities()`, `probe()`
- **Library:** `library_list()`, `library_get()`, `library_scan()`, `library_refresh()`, `library_browse()`, `library_empty_trash()`
- **Media:** `media_search()`, `media_get()`
- **Sessions:** `session_list()`, `session_terminate()`, `session_history()`
- **Playlists:** `playlist_list()`, `playlist_get()`, `playlist_create()`, `playlist_delete()`
- **Metadata:** `metadata_delete()`, `metadata_edit()`, `metadata_refresh()`
- **Hubs:** `hubs_continue_watching()`
- **Butler:** `butler_list()`, `butler_run()`
- **Scrobbling:** `item_scrobble()`, `item_unscrobble()`
- **Updates:** `updater_status()`

All methods use `HttpClient` from `lab_apis::core`. Errors are typed as `PlexError`, which wraps `ApiError` transparently.

### Dispatch Layer (`crates/lab/src/dispatch/plex/`)

Structure follows the standard layout:

- **catalog.rs:** ActionSpec definitions for all 31 actions (29 service + `help` + `schema`)
- **client.rs:** `client_from_env()` and `require_client()` for env-based client construction
- **params.rs:** Parameter extraction helpers (require_key, optional_string, etc.)
- **dispatch.rs:** `dispatch(action, params)` and `dispatch_with_client(client, action, params)` routing
  - Unmarked actions: `help`, `schema`
  - Marked destructive: `library.refresh`, `session.terminate`, `playlist.create`, `playlist.delete`, `library.empty-trash`, `metadata.delete`

All 29 service actions wire through `dispatch_with_client()`, which calls the corresponding SDK method and serializes results to JSON.

### CLI (`crates/lab/src/cli/plex.rs`)

Thin shim using generic action dispatch:

```bash
lab plex help
lab plex library.list
lab plex media.search --params '{"query":"The Matrix"}'
lab plex session.terminate --params '{"session_id":"123"}' --yes
```

- Supports `--json` for JSON output (default: human table)
- Supports `--verbose` for debug logs (via `tracing`)
- Respects `-y` / `--yes` for destructive operations
- Respects `--dry-run` for dry-run mode

### MCP (`crates/lab/src/mcp/services/plex.rs`)

Thin bridge that delegates to dispatch layer. Registered as one tool `plex` with:

```json
{
  "name": "plex",
  "description": "Plex Media Server control",
  "inputSchema": {
    "type": "object",
    "properties": {
      "action": { "type": "string", "description": "Action to run (e.g., library.list)" },
      "params": { "type": "object", "description": "Action-specific parameters" }
    },
    "required": ["action"]
  }
}
```

- `help` action returns the action catalog
- `schema` action returns the parameter schema for a named action
- Destructive actions (`library.refresh`, `session.terminate`, etc.) require elicitation confirmation from the client
- All actions return `Result<Value, ToolError>` with structured error envelopes

### API (`crates/lab/src/api/services/plex.rs`)

HTTP route group at `POST /v1/plex`:

```bash
curl -X POST http://localhost:8080/v1/plex \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"action":"library.list"}'

curl -X POST http://localhost:8080/v1/plex \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"action":"session.terminate","params":{"session_id":"123","confirm":true}}'
```

- Handlers call `dispatch_with_client()` with pre-built client from `AppState`
- Destructive actions require `"confirm": true` in the JSON request body
- All errors return structured JSON envelopes with `kind` field mapping to HTTP status

## OpenAPI Endpoint Inventory

The table below tracks the full OpenAPI spec surface. Rows updated to ✅ where the SDK method backing the implemented action above matches the endpoint.

## Activities

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /activities | listActivities | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /activities/{activityId} | cancelActivity | ⬜ | ⬜ | ⬜ | ⬜ |

## Butler

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /butler | stopTasks | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /butler | getTasks / `butler_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /butler | startTasks | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /butler/{butlerTask} | stopTask | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /butler/{butlerTask} | startTask / `butler_run` | ✅ | ✅ | ✅ | ✅ |

## Collections

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /library/collections | createCollection | ⬜ | ⬜ | ⬜ | ⬜ |

## Content

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /library/collections/{collectionId}/composite/{updatedAt} | getCollectionImage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/collections/{collectionId}/items | getCollectionItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids} | getMetadataItem / `media_get` | ✅ | ✅ | ✅ | ✅ |
| GET | /library/sections/{sectionId}/albums | getAlbums | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/all | listContent / `library_browse` | ✅ | ✅ | ✅ | ✅ |
| GET | /library/sections/{sectionId}/allLeaves | getAllLeaves | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/arts | getArts | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/categories | getCategories | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/cluster | getCluster | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/computePath | getSonicPath | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/location | getFolders | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/moment | listMoments | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/nearest | getSonicallySimilar | ⬜ | ⬜ | ⬜ | ⬜ |

## Devices

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /media/grabbers | getAvailableGrabbers | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/grabbers/devices | listDevices | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/grabbers/devices | addDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/grabbers/devices/discover | discoverDevices | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/grabbers/devices/{deviceId} | removeDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/grabbers/devices/{deviceId} | getDeviceDetails | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /media/grabbers/devices/{deviceId} | modifyDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /media/grabbers/devices/{deviceId}/channelmap | setChannelmap | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/grabbers/devices/{deviceId}/channels | getDevicesChannels | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /media/grabbers/devices/{deviceId}/prefs | setDevicePreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/grabbers/devices/{deviceId}/scan | stopScan | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/grabbers/devices/{deviceId}/scan | scan | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/grabbers/devices/{deviceId}/thumb/{version} | getThumb | ⬜ | ⬜ | ⬜ | ⬜ |

## Download Queue

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /downloadQueue | createDownloadQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /downloadQueue/{queueId} | getDownloadQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /downloadQueue/{queueId}/add | addDownloadQueueItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /downloadQueue/{queueId}/item/{itemId}/decision | getItemDecision | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /downloadQueue/{queueId}/item/{itemId}/media | getDownloadQueueMedia | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /downloadQueue/{queueId}/items | listDownloadQueueItems | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /downloadQueue/{queueId}/items/{itemId} | removeDownloadQueueItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /downloadQueue/{queueId}/items/{itemId} | getDownloadQueueItems | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /downloadQueue/{queueId}/items/{itemId}/restart | restartProcessingDownloadQueueItems | ⬜ | ⬜ | ⬜ | ⬜ |

## DVRs

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /livetv/dvrs | listDVRs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /livetv/dvrs | createDVR | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /livetv/dvrs/{dvrId} | deleteDVR | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/dvrs/{dvrId} | getDVR | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /livetv/dvrs/{dvrId}/channels/{channel}/tune | tuneChannel | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /livetv/dvrs/{dvrId}/devices/{deviceId} | removeDeviceFromDVR | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /livetv/dvrs/{dvrId}/devices/{deviceId} | addDeviceToDVR | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /livetv/dvrs/{dvrId}/lineups | deleteLineup | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /livetv/dvrs/{dvrId}/lineups | addLineup | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /livetv/dvrs/{dvrId}/prefs | setDVRPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /livetv/dvrs/{dvrId}/reloadGuide | stopDVRReload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /livetv/dvrs/{dvrId}/reloadGuide | reloadGuide | ⬜ | ⬜ | ⬜ | ⬜ |

## EPG

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /livetv/epg/channelmap | computeChannelMap | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/channels | getChannels | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/countries | getCountries | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/countries/{country}/{epgId}/lineups | getCountriesLineups | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/countries/{country}/{epgId}/regions | getCountryRegions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/countries/{country}/{epgId}/regions/{region}/lineups | listLineups | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/languages | getAllLanguages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/lineup | getLineup | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/epg/lineupchannels | getLineupChannels | ⬜ | ⬜ | ⬜ | ⬜ |

## Events

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /:/eventsource/notifications | getNotifications | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /:/websocket/notifications | connectWebSocket | ⬜ | ⬜ | ⬜ | ⬜ |

## General

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | / | getServerInfo / `server_info` | ✅ | ✅ | ✅ | ✅ |
| GET | /identity | getIdentity | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /security/resources | getSourceConnectionInformation | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /security/token | getTransientToken | ⬜ | ⬜ | ⬜ | ⬜ |

## Hubs

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /hubs | getAllHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/continueWatching | getContinueWatching / `hubs_continue_watching` | ✅ | ✅ | ✅ | ✅ |
| GET | /hubs/items | getHubItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/metadata/{metadataId} | getMetadataHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/metadata/{metadataId}/postplay | getPostplayHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/metadata/{metadataId}/related | getRelatedHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/promoted | getPromotedHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/sections/{sectionId} | getSectionHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /hubs/sections/{sectionId}/manage | resetSectionDefaults | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /hubs/sections/{sectionId}/manage | listHubs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /hubs/sections/{sectionId}/manage | createCustomHub | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /hubs/sections/{sectionId}/manage/move | moveHub | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /hubs/sections/{sectionId}/manage/{identifier} | deleteCustomHub | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /hubs/sections/{sectionId}/manage/{identifier} | updateHubVisibility | ⬜ | ⬜ | ⬜ | ⬜ |

## Library

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /library/all | getLibraryItems | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/caches | deleteCaches | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/clean/bundles | cleanBundles | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/file | ingestTransientItem | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/matches | getLibraryMatches | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/media/{mediaId}/chapterImages/{chapter} | getChapterImage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/augmentations/{augmentationId} | getAugmentationStatus | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/metadata/{ids} | deleteMetadataItem / `metadata_delete` | ✅ | ✅ | ✅ | ✅ |
| PUT | /library/metadata/{ids} | editMetadataItem / `metadata_edit` | ✅ | ✅ | ✅ | ✅ |
| PUT | /library/metadata/{ids}/addetect | detectAds | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/allLeaves | getAllItemLeaves | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/analyze | analyzeMetadata | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/chapterThumbs | generateThumbs | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/credits | detectCredits | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/extras | getExtras | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/metadata/{ids}/extras | addExtras | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/file | getFile | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/index | startBifGeneration | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/intro | detectIntros | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/metadata/{ids}/marker | createMarker | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/metadata/{ids}/marker/{marker} | deleteMarker | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/marker/{marker} | editMarker | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/match | matchItem | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/matches | listMatches | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/metadata/{ids}/media/{mediaItem} | deleteMediaItem | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/merge | mergeItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/nearest | listSonicallySimilar | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/prefs | setItemPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/refresh | refreshItemsMetadata / `metadata_refresh` | ✅ | ✅ | ✅ | ✅ |
| GET | /library/metadata/{ids}/related | getRelatedItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/similar | listSimilar | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/split | splitItem | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/subtitles | addSubtitles | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/tree | getItemTree | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/unmatch | unmatch | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/users/top | listTopUsers | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/voiceActivity | detectVoiceActivity | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/metadata/{ids}/{element} | setItemArtwork | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/metadata/{ids}/{element} | updateItemArtwork | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/metadata/{ids}/{element}/{timestamp} | getItemArtwork | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/optimize | optimizeDatabase | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/parts/{partId} | setStreamSelection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/parts/{partId}/indexes/{index} | getPartIndex | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/parts/{partId}/indexes/{index}/{offset} | getImageFromBif | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/parts/{partId}/{changestamp}/{filename} | getMediaPart | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/people/{personId} | getPerson | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/people/{personId}/media | listPersonMedia | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/randomArtwork | getRandomArtwork | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/all | getSections / `library_list` (calls /library/sections) | ✅ | ✅ | ✅ | ✅ |
| POST | /library/sections/all | addSection | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/all/refresh | stopAllRefreshes | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/prefs | getSectionsPrefs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/sections/refresh | refreshSectionsMetadata | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/{sectionId} | deleteLibrarySection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId} | getLibraryDetails / `library_get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /library/sections/{sectionId} | editSection | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/sections/{sectionId}/all | updateItems | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/sections/{sectionId}/analyze | startAnalysis | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/autocomplete | autocomplete | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/{sectionId}/collection/{collectionId} | deleteCollection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/collections | getCollections | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/common | getCommon | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/composite/{updatedAt} | getSectionImage | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/sections/{sectionId}/emptyTrash | emptyTrash / `library_empty_trash` | ✅ | ✅ | ✅ | ✅ |
| GET | /library/sections/{sectionId}/filters | getSectionFilters | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/firstCharacters | getFirstCharacters | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/{sectionId}/indexes | deleteIndexes | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/{sectionId}/intros | deleteIntros | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/sections/{sectionId}/prefs | getSectionPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/sections/{sectionId}/prefs | setSectionPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/sections/{sectionId}/refresh | cancelRefresh | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /library/sections/{sectionId}/refresh | refreshSection / `library_scan` (GET) / `library_refresh` (GET+force=1) | ✅ | ✅ | ✅ | ✅ |
| GET | /library/sections/{sectionId}/sorts | getAvailableSorts | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /library/streams/{streamId}.{ext} | deleteStream | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/streams/{streamId}.{ext} | getStream | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/streams/{streamId}.{ext} | setStreamOffset | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/streams/{streamId}/levels | getStreamLevels | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/streams/{streamId}/loudness | getStreamLoudness | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /library/tags | getTags | ⬜ | ⬜ | ⬜ | ⬜ |

## Library Collections

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| PUT | /library/collections/{collectionId}/items | addCollectionItems | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/collections/{collectionId}/items/{itemId} | deleteCollectionItem | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /library/collections/{collectionId}/items/{itemId}/move | moveCollectionItem | ⬜ | ⬜ | ⬜ | ⬜ |

## Library Playlists

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /playlists | createPlaylist / `playlist_create` | ✅ | ✅ | ✅ | ✅ |
| POST | /playlists/upload | uploadPlaylist | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /playlists/{playlistId} | deletePlaylist / `playlist_delete` | ✅ | ✅ | ✅ | ✅ |
| PUT | /playlists/{playlistId} | updatePlaylist | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /playlists/{playlistId}/generators | getPlaylistGenerators | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /playlists/{playlistId}/items | clearPlaylistItems | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playlists/{playlistId}/items | addPlaylistItems | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /playlists/{playlistId}/items/{generatorId} | deletePlaylistItem | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /playlists/{playlistId}/items/{generatorId} | getPlaylistGenerator | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playlists/{playlistId}/items/{generatorId} | modifyPlaylistGenerator | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /playlists/{playlistId}/items/{generatorId}/items | getPlaylistGeneratorItems | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playlists/{playlistId}/items/{generatorId}/{metadataId}/{action} | refreshPlaylist | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playlists/{playlistId}/items/{playlistItemId}/move | movePlaylistItem | ⬜ | ⬜ | ⬜ | ⬜ |

## Live TV

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /livetv/sessions | getSessions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/sessions/{sessionId} | getLiveTVSession | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/sessions/{sessionId}/{consumerId}/index.m3u8 | getSessionPlaylistIndex | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /livetv/sessions/{sessionId}/{consumerId}/{segmentId} | getSessionSegment | ⬜ | ⬜ | ⬜ | ⬜ |

## Log

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /log | writeLog | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /log | writeMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /log/networked | enablePapertrail | ⬜ | ⬜ | ⬜ | ⬜ |

## Play Queue

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /playQueues | createPlayQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /playQueues/{playQueueId} | getPlayQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playQueues/{playQueueId} | addToPlayQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /playQueues/{playQueueId}/items | clearPlayQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /playQueues/{playQueueId}/items/{playQueueItemId} | deletePlayQueueItem | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playQueues/{playQueueId}/items/{playQueueItemId}/move | movePlayQueueItem | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playQueues/{playQueueId}/reset | resetPlayQueue | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playQueues/{playQueueId}/shuffle | shuffle | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /playQueues/{playQueueId}/unshuffle | unshuffle | ⬜ | ⬜ | ⬜ | ⬜ |

## Playlist

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /playlists | listPlaylists / `playlist_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /playlists/{playlistId} | getPlaylist / `playlist_get` | ✅ | ✅ | ✅ | ✅ |
| GET | /playlists/{playlistId}/items | getPlaylistItems | ⬜ | ⬜ | ⬜ | ⬜ |

## Preferences

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /:/prefs | getAllPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /:/prefs | setPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /:/prefs/get | getPreference | ⬜ | ⬜ | ⬜ | ⬜ |

## Provider

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /media/providers | listProviders / `server_capabilities` | ✅ | ✅ | ✅ | ✅ |
| POST | /media/providers | addProvider | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/providers/refresh | refreshProviders | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/providers/{provider} | deleteMediaProvider | ⬜ | ⬜ | ⬜ | ⬜ |

## Rate

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| PUT | /:/rate | setRating | ⬜ | ⬜ | ⬜ | ⬜ |

## Search

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /hubs/search | searchHubs / `media_search` | ✅ | ✅ | ✅ | ✅ |
| GET | /hubs/search/voice | voiceSearchHubs | ⬜ | ⬜ | ⬜ | ⬜ |

## Status

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /status/sessions | listSessions / `session_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /status/sessions/background | getBackgroundTasks | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /status/sessions/history/all | listPlaybackHistory / `session_history` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /status/sessions/history/{historyId} | deleteHistory | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /status/sessions/history/{historyId} | getHistoryItem | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /status/sessions/terminate | terminateSession / `session_terminate` (GET) | ✅ | ✅ | ✅ | ✅ |

## Subscriptions

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /media/grabbers/operations/{operationId} | cancelGrab | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/subscriptions | getAllSubscriptions | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/subscriptions | createSubscription | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/subscriptions/process | processSubscriptions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/subscriptions/scheduled | getScheduledRecordings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/subscriptions/template | getTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/subscriptions/{subscriptionId} | deleteSubscription | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/subscriptions/{subscriptionId} | getSubscription | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /media/subscriptions/{subscriptionId} | editSubscriptionPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /media/subscriptions/{subscriptionId}/move | reorderSubscription | ⬜ | ⬜ | ⬜ | ⬜ |

## Timeline

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| PUT | /:/scrobble | markPlayed / `item_scrobble` (GET quirk) | ✅ | ✅ | ✅ | ✅ |
| POST | /:/timeline | report | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /:/unscrobble | unscrobble / `item_unscrobble` (GET quirk) | ✅ | ✅ | ✅ | ✅ |

## Transcoder

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /photo/:/transcode | transcodeImage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{transcodeType}/:/transcode/universal/decision | makeDecision | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /{transcodeType}/:/transcode/universal/fallback | triggerFallback | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{transcodeType}/:/transcode/universal/start.{extension} | startTranscodeSession | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /{transcodeType}/:/transcode/universal/subtitles | transcodeSubtitles | ⬜ | ⬜ | ⬜ | ⬜ |

## UltraBlur

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /services/ultrablur/colors | getColors | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /services/ultrablur/image | getImage | ⬜ | ⬜ | ⬜ | ⬜ |

## Updater

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| PUT | /updater/apply | applyUpdates | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /updater/check | checkUpdates | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /updater/status | getUpdatesStatus / `updater_status` | ✅ | ✅ | ✅ | ✅ |

## Authentication

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /user | getTokenDetails | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/signin | post-users-sign-in-data | ⬜ | ⬜ | ⬜ | ⬜ |

## Users

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /users | get-users | ⬜ | ⬜ | ⬜ | ⬜ |

## Plex

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /resources | get-server-resources | ⬜ | ⬜ | ⬜ | ⬜ |
