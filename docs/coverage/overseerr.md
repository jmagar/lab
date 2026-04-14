# Overseerr API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/api-specs/overseerr.openapi.yaml
**OpenAPI version:** 3.0.2
**API version:** 1.0.0
**Paths:** 170
**Servers:** {server}/api/v1
**Security schemes:** `X-Api-Key` header (env var: `OVERSEERR_API_KEY`)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented across all surfaces (SDK client, MCP, CLI, HTTP API) |
| ⬜ | Not implemented; row is spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Implemented Actions

These actions are fully wired: `lab-apis` client method → dispatch layer → MCP + CLI + HTTP API.
The dispatch action strings are the canonical identifiers; the SDK method names are the underlying Rust methods.

| Action | HTTP Method | Endpoint | SDK Method | MCP | CLI | API |
|--------|-------------|----------|------------|-----|-----|-----|
| `health` | GET | /api/v1/status | `probe()` | ✅ | ✅ | ✅ |
| `status` | GET | /api/v1/status | `status()` | ✅ | ✅ | ✅ |
| `request.list` | GET | /api/v1/request | `request_list()` | ✅ | ✅ | ✅ |
| `request.get` | GET | /api/v1/request/{id} | `request_get()` | ✅ | ✅ | ✅ |
| `request.create` | POST | /api/v1/request | `request_create()` | ✅ | ✅ | ✅ |
| `request.approve` | POST | /api/v1/request/{id}/approve | `request_approve()` | ✅ | ✅ | ✅ |
| `request.decline` | POST | /api/v1/request/{id}/decline | `request_decline()` | ✅ | ✅ | ✅ |
| `request.delete` | DELETE | /api/v1/request/{id} | `request_delete()` | ✅ | ✅ | ✅ |
| `request.retry` | POST | /api/v1/request/{id}/retry | `request_retry()` | ✅ | ✅ | ✅ |
| `request.count` | GET | /api/v1/request/count | `request_count()` | ✅ | ✅ | ✅ |
| `movie.search` | GET | /api/v1/search | `search()` | ✅ | ✅ | ✅ |
| `tv.search` | GET | /api/v1/search | `search()` | ✅ | ✅ | ✅ |
| `movie.get` | GET | /api/v1/movie/{tmdb_id} | `movie_get()` | ✅ | ✅ | ✅ |
| `tv.get` | GET | /api/v1/tv/{tmdb_id} | `tv_get()` | ✅ | ✅ | ✅ |
| `user.list` | GET | /api/v1/user | `user_list()` | ✅ | ✅ | ✅ |
| `user.get` | GET | /api/v1/user/{id} | `user_get()` | ✅ | ✅ | ✅ |
| `user.requests` | GET | /api/v1/user/{id}/requests | `user_requests()` | ✅ | ✅ | ✅ |
| `user.quota` | GET | /api/v1/user/{id}/quota | `user_quota()` | ✅ | ✅ | ✅ |
| `user.edit` | PUT | /api/v1/user/{id} | `user_edit()` | ✅ | ✅ | ✅ |
| `issue.list` | GET | /api/v1/issue | `issue_list()` | ✅ | ✅ | ✅ |
| `issue.get` | GET | /api/v1/issue/{id} | `issue_get()` | ✅ | ✅ | ✅ |
| `issue.create` | POST | /api/v1/issue | `issue_create()` | ✅ | ✅ | ✅ |
| `issue.comment` | POST | /api/v1/issue/{id}/comment | `issue_comment()` | ✅ | ✅ | ✅ |
| `issue.update` | POST | /api/v1/issue/{id}/{status} | `issue_update()` | ✅ | ✅ | ✅ |
| `media.delete` | DELETE | /api/v1/media/{id} | `media_delete()` | ✅ | ✅ | ✅ |
| `media.update-status` | POST | /api/v1/media/{id}/{status} | `media_update_status()` | ✅ | ✅ | ✅ |
| `job.run` | POST | /api/v1/settings/jobs/{id}/run | `job_run()` | ✅ | ✅ | ✅ |
| `discover.trending` | GET | /api/v1/discover/trending | `discover_trending()` | ✅ | ✅ | ✅ |
| `help` | - | - | - | ✅ | ✅ | ✅ |
| `schema` | - | - | - | ✅ | ✅ | ✅ |

## Config / Auth

| Env Var | Required | Description |
|---------|----------|-------------|
| `OVERSEERR_URL` | yes | Base URL (e.g. `https://overseerr.example.com`) |
| `OVERSEERR_API_KEY` | yes | API key sent as `X-Api-Key` header |

## CLI Usage

The CLI is a flat action+params shim (`OverseerrArgs { action, params }`). There are no typed subcommands.

```bash
lab overseerr                                            # shows help
lab overseerr status
lab overseerr health
lab overseerr request.list --params '{"filter":"pending"}'
lab overseerr request.get --params '{"id":42}'
lab overseerr request.create --params '{"media_type":"movie","media_id":603}'
lab overseerr request.approve --params '{"id":42}'
lab overseerr request.decline --params '{"id":42}'
lab overseerr request.delete --params '{"id":42,"confirm":true}'
lab overseerr request.retry --params '{"id":42}'
lab overseerr request.count
lab overseerr movie.search --params '{"query":"The Matrix"}'
lab overseerr tv.search --params '{"query":"Breaking Bad"}'
lab overseerr movie.get --params '{"tmdb_id":603}'
lab overseerr tv.get --params '{"tmdb_id":1396}'
lab overseerr user.list
lab overseerr user.get --params '{"id":1}'
lab overseerr user.requests --params '{"id":1}'
lab overseerr user.quota --params '{"id":1}'
lab overseerr user.edit --params '{"id":1,"body":{"email":"new@example.com"}}'
lab overseerr issue.list
lab overseerr issue.get --params '{"id":5}'
lab overseerr issue.create --params '{"issue_type":1,"message":"Video stutters","media_id":42}'
lab overseerr issue.comment --params '{"id":5,"message":"Still happening"}'
lab overseerr issue.update --params '{"id":5,"status":"resolved"}'
lab overseerr media.delete --params '{"id":42,"confirm":true}'
lab overseerr media.update-status --params '{"id":42,"status":"available"}'
lab overseerr job.run --params '{"id":"plex-recently-added-scan"}'
lab overseerr discover.trending
lab overseerr schema --params '{"action":"request.create"}'
```

## Action Details

### Built-in Actions

#### `help`
- **Params:** none
- **Returns:** Catalog of all available actions with descriptions and parameter specs
- **Notes:** Available on every tool. Shown when no action is specified.

#### `schema`
- **Params:**
  - `action` (string, required) — action name to describe
- **Returns:** Parameter schema for the specified action (types and required flags)
- **Notes:** Available on every tool. Returns `unknown_action` if the action doesn't exist.

### Service Actions

#### `health`
- **Params:** none
- **Returns:** `{ "ok": true }`
- **Notes:** Fetches `/api/v1/status` and discards the body; lightweight connectivity probe.

### `status`
- **Params:** none
- **Returns:** `OverseerrStatus` — `{ "version", "commitTag"?, "updateAvailable", "commitsBehind"?, "restartRequired" }`

### `request.list`
- **Params:**
  - `take` (integer, optional) — results to return (default 20)
  - `skip` (integer, optional) — results to skip for pagination
  - `filter` (string, optional) — `all|approved|available|pending|processing|unavailable|failed`
  - `sort` (string, optional) — `added|modified`
  - `requested_by` (integer, optional) — filter by requesting user ID
- **Returns:** `RequestList` — `{ "pageInfo": {...}, "results": [MediaRequest,...] }`

### `request.get`
- **Params:** `id` (integer, required)
- **Returns:** `MediaRequest`

### `request.create`
- **Params:**
  - `media_type` (string, required) — `movie` or `tv`
  - `media_id` (integer, required) — TMDB media ID
  - `seasons` (string, optional) — comma-separated season numbers for TV (e.g. `"1,2,3"`)
  - `is4k` (bool, optional) — request 4K quality (default false)
- **Returns:** `MediaRequest`
- **Notes:** `seasons` is parsed from a comma-separated string into `Vec<u32>` by the dispatch layer.

### `request.approve`
- **Params:** `id` (integer, required)
- **Returns:** `MediaRequest`

### `request.decline`
- **Params:** `id` (integer, required)
- **Returns:** `MediaRequest`

### `request.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `{ "deleted": true }`
- **Notes:** Destructive operation. Requires `"confirm": true` in params (MCP/HTTP/CLI). CLI will prompt interactively without `-y`/`--yes` flag.

### `movie.search` / `tv.search`
- **Params:**
  - `query` (string, required) — search terms
  - `page` (integer, optional) — page number (default 1)
- **Returns:** `SearchResponse` — `{ "page", "totalPages", "totalResults", "results": [SearchResult,...] }`
- **Notes:** Both actions call the same underlying `/api/v1/search` endpoint and `search()` client method.
  The `movie.search` / `tv.search` distinction is at the dispatch layer only; Overseerr returns both
  movie and TV results from a single search endpoint.

### `movie.get`
- **Params:** `tmdb_id` (integer, required)
- **Returns:** `MovieDetail` — `{ "id", "imdbId"?, "title", "originalTitle"?, "overview"?, "posterPath"?, "backdropPath"?, "releaseDate"?, "runtime"?, "voteAverage"?, "voteCount"?, "genres"?, "mediaInfo"? }`

### `tv.get`
- **Params:** `tmdb_id` (integer, required)
- **Returns:** `TvDetail` — `{ "id", "name", "originalName"?, "overview"?, "posterPath"?, "backdropPath"?, "firstAirDate"?, "voteAverage"?, "voteCount"?, "numberOfSeasons"?, "genres"?, "mediaInfo"? }`

### `user.list`
- **Params:**
  - `take` (integer, optional)
  - `skip` (integer, optional)
- **Returns:** `UserList` — `{ "pageInfo"?: {...}, "results": [User,...] }`

### `user.get`
- **Params:** `id` (integer, required)
- **Returns:** `User` — `{ "id", "email", "username"?, "plexUsername"?, "userType"?, "permissions"?, "avatar"?, "createdAt", "updatedAt", "requestCount"? }`

### `issue.list`
- **Params:**
  - `take` (integer, optional)
  - `skip` (integer, optional)
  - `filter` (string, optional) — `open|resolved`
  - `sort` (string, optional) — `added|modified`
- **Returns:** `IssueList` — `{ "pageInfo"?: {...}, "results": [Issue,...] }`

### `issue.get`
- **Params:** `id` (integer, required)
- **Returns:** `Issue` — `{ "id", "issueType", "status", "problemSeason"?, "problemEpisode"?, "media"?, "createdBy"?, "modifiedBy"?, "comments"?: [IssueComment,...], "createdAt", "updatedAt" }`

### `issue.create`
- **Params:**
  - `issue_type` (integer, required) — 1=video, 2=audio, 3=subtitle, 4=other
  - `message` (string, required) — description of the issue
  - `media_id` (integer, required) — Overseerr media ID (not TMDB ID)
  - `problem_season` (integer, optional) — season number (TV only)
  - `problem_episode` (integer, optional) — episode number (TV only)
- **Returns:** `Issue`

### `issue.comment`
- **Params:**
  - `id` (integer, required) — issue ID
  - `message` (string, required) — comment text
- **Returns:** `IssueComment` — `{ "id", "message", "user"?, "createdAt", "updatedAt" }`

### `request.retry`
- **Params:** `id` (integer, required)
- **Returns:** `MediaRequest`
- **Notes:** Retries a failed request by resubmitting it to the downstream service.

### `request.count`
- **Params:** none
- **Returns:** `RequestCount` — `{ "pending", "approved", "declined", "processing", "available", "total": u32 }`

### `issue.update`
- **Params:**
  - `id` (integer, required) — issue ID
  - `status` (string, required) — `resolved` or `open`
- **Returns:** `Issue`
- **Notes:** Status is a path parameter, not a body field.

### `media.delete` *(destructive)*
- **Params:** `id` (integer, required) — Overseerr media ID (not TMDB ID)
- **Returns:** `{ "deleted": true }`
- **Notes:** Destructive operation. Requires `"confirm": true` in params (MCP/HTTP/CLI). CLI will prompt interactively without `-y`/`--yes` flag.

### `media.update-status`
- **Params:**
  - `id` (integer, required) — Overseerr media ID
  - `status` (string, required) — new status string (e.g. `available`, `unknown`, `blacklisted`)
- **Returns:** `{ "ok": true }`
- **Notes:** Status is a path parameter passed directly to the upstream API.

### `user.requests`
- **Params:** `id` (integer, required) — user ID
- **Returns:** `RequestList` — `{ "pageInfo"?: {...}, "results": [MediaRequest,...] }`

### `user.quota`
- **Params:** `id` (integer, required) — user ID
- **Returns:** `UserQuota` — `{ "movie": {...}, "tv": {...} }` (free-form JSON; shape varies)

### `user.edit`
- **Params:**
  - `id` (integer, required) — user ID
  - `body` (object, required) — partial user fields to update (any subset of User fields)
- **Returns:** `User`
- **Notes:** Uses a free-form JSON body (`serde_json::Value`) since the full User struct is complex.

### `job.run`
- **Params:** `id` (string, required) — job ID (e.g. `plex-recently-added-scan`, `radarr-scan`)
- **Returns:** job status JSON (free-form `serde_json::Value`)

### `discover.trending`
- **Params:** none
- **Returns:** discover page JSON (free-form `serde_json::Value`) — `{ "page", "totalPages", "totalResults", "results": [...] }`

## Not Yet Implemented

All other Overseerr API paths. Grouped by spec tag for reference.

### public

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /status/appdata | ⬜ | ⬜ | ⬜ | ⬜ |

### settings

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /settings/about | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/cache | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/cache/{cacheId}/flush | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/discover | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/discover | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/discover/add | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/discover/reset | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/discover/{sliderId} | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/discover/{sliderId} | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/initialize | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/jobs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/jobs/{jobId}/cancel | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/jobs/{jobId}/run | ✅ | ✅ | ✅ | ✅ |
| POST | /settings/jobs/{jobId}/schedule | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/logs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/main | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/main | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/main/regenerate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET/POST | /settings/notifications/* | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/plex | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/devices/servers | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/library | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/sync | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/plex/sync | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/users | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/public | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/radarr | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/radarr | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/radarr/test | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/radarr/{radarrId} | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/radarr/{radarrId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/radarr/{radarrId}/profiles | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/sonarr | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/sonarr | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/sonarr/test | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/sonarr/{sonarrId} | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/sonarr/{sonarrId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/tautulli | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/tautulli | ⬜ | ⬜ | ⬜ | ⬜ |

### auth

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| POST | /auth/local | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/logout | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /auth/me | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/plex | ⬜ | ⬜ | ⬜ | ⬜ |

### users (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| POST | /auth/reset-password | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/reset-password/{guid} | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /user | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/import-from-plex | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/registerPushSubscription | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /user/{userId} | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /user/{userId} | ✅ | ✅ | ✅ | ✅ |
| DELETE | /user/{userId}/pushSubscription/{endpoint} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/pushSubscription/{endpoint} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/pushSubscriptions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/quota | ✅ | ✅ | ✅ | ✅ |
| GET | /user/{userId}/requests | ✅ | ✅ | ✅ | ✅ |
| GET | /user/{userId}/settings/main | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/main | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/notifications | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/notifications | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/password | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/password | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/permissions | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/permissions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/watch_data | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/watchlist | ⬜ | ⬜ | ⬜ | ⬜ |

### search / discover (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /discover/genreslider/movie | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/genreslider/tv | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/keyword/{keywordId}/movies | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/genre/{genreId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/language/{language} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/studio/{studioId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/upcoming | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/trending | ✅ | ✅ | ✅ | ✅ |
| GET | /discover/tv | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/genre/{genreId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/language/{language} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/network/{networkId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/upcoming | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/watchlist | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /search/company | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /search/keyword | ⬜ | ⬜ | ⬜ | ⬜ |

### request (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /request/count | ✅ | ✅ | ✅ | ✅ |
| PUT | /request/{requestId} | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /request/{requestId}/retry | ✅ | ✅ | ✅ | ✅ |

### movies (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /movie/{movieId}/ratings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/ratingscombined | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/recommendations | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/similar | ⬜ | ⬜ | ⬜ | ⬜ |

### tv (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /tv/{tvId}/ratings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/recommendations | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/season/{seasonId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/similar | ⬜ | ⬜ | ⬜ | ⬜ |

### other

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /keyword/{keywordId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/movies | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/regions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/tv | ⬜ | ⬜ | ⬜ | ⬜ |

### person

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /person/{personId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /person/{personId}/combined_credits | ⬜ | ⬜ | ⬜ | ⬜ |

### media

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /media | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/{mediaId} | ✅ | ✅ | ✅ | ✅ |
| GET | /media/{mediaId}/watch_data | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/{mediaId}/{status} | ✅ | ✅ | ✅ | ✅ |

### collection

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /collection/{collectionId} | ⬜ | ⬜ | ⬜ | ⬜ |

### service

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /service/radarr | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/radarr/{radarrId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr/lookup/{tmdbId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr/{sonarrId} | ⬜ | ⬜ | ⬜ | ⬜ |

### tmdb (metadata)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /backdrops | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /genres/movie | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /genres/tv | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /languages | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /network/{networkId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /regions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /studio/{studioId} | ⬜ | ⬜ | ⬜ | ⬜ |

### issue (extended)

| Method | Endpoint | Impl | MCP | CLI | API |
|--------|----------|------|-----|-----|-----|
| GET | /issue/count | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /issue/{issueId} | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /issue/{issueId}/{status} | ✅ | ✅ | ✅ | ✅ |
| DELETE | /issueComment/{commentId} | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /issueComment/{commentId} | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /issueComment/{commentId} | ⬜ | ⬜ | ⬜ | ⬜ |
