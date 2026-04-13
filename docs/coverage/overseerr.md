# Overseerr API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** docs/api-specs/overseerr.openapi.yaml
**OpenAPI version:** 3.0.2
**API version:** 1.0.0
**Paths:** 170
**Servers:** {server}/api/v1
**Security schemes:** cookieAuth, apiKey

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## public

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /status | `status()` | ✅ | ✅ | ✅ | ✅ |
| GET | /status/appdata | - | ⬜ | ⬜ | ⬜ | ⬜ |

## settings

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /settings/about | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/cache | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/cache/{cacheId}/flush | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/discover | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/discover | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/discover/add | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/discover/reset | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/discover/{sliderId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/discover/{sliderId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/initialize | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/jobs | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/jobs/{jobId}/cancel | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/jobs/{jobId}/run | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/jobs/{jobId}/schedule | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/logs | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/main | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/main | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/main/regenerate | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/discord | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/discord | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/discord/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/email | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/email | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/email/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/gotify | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/gotify | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/gotify/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/lunasea | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/lunasea | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/lunasea/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/pushbullet | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/pushbullet | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/pushbullet/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/pushover | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/pushover | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/pushover/sounds | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/pushover/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/slack | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/slack | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/slack/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/telegram | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/telegram | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/telegram/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/webhook | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/webhook | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/webhook/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/notifications/webpush | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/webpush | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/notifications/webpush/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/plex | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/devices/servers | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/library | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/sync | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/plex/sync | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/plex/users | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/public | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/radarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/radarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/radarr/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/radarr/{radarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/radarr/{radarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/radarr/{radarrId}/profiles | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/sonarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/sonarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/sonarr/test | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /settings/sonarr/{sonarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /settings/sonarr/{sonarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/tautulli | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/tautulli | - | ⬜ | ⬜ | ⬜ | ⬜ |

## auth

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /auth/local | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/logout | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /auth/me | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/plex | - | ⬜ | ⬜ | ⬜ | ⬜ |

## users

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /auth/reset-password | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/reset-password/{guid} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user | `user_list()` | ✅ | ✅ | ✅ | ✅ |
| POST | /user | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /user | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/import-from-plex | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/registerPushSubscription | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /user/{userId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId} | `user_get()` | ✅ | ✅ | ✅ | ✅ |
| PUT | /user/{userId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /user/{userId}/pushSubscription/{endpoint} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/pushSubscription/{endpoint} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/pushSubscriptions | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/quota | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/requests | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/main | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/main | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/notifications | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/notifications | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/password | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/password | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/settings/permissions | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user/{userId}/settings/permissions | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/watch_data | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user/{userId}/watchlist | - | ⬜ | ⬜ | ⬜ | ⬜ |

## search

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /discover/genreslider/movie | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/genreslider/tv | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/keyword/{keywordId}/movies | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/genre/{genreId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/language/{language} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/studio/{studioId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/movies/upcoming | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/trending | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/genre/{genreId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/language/{language} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/network/{networkId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/tv/upcoming | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /discover/watchlist | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /search | `search()` | ✅ | ✅ | ✅ | ✅ |
| GET | /search/company | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /search/keyword | - | ⬜ | ⬜ | ⬜ | ⬜ |

## request

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /request | `request_list()` | ✅ | ✅ | ✅ | ✅ |
| POST | /request | `request_create()` | ✅ | ✅ | ✅ | ✅ |
| GET | /request/count | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /request/{requestId} | `request_delete()` | ✅ | ✅ | ✅ | ✅ |
| GET | /request/{requestId} | `request_get()` | ✅ | ✅ | ✅ | ✅ |
| PUT | /request/{requestId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /request/{requestId}/retry | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /request/{requestId}/{status} | `request_approve()`/`request_decline()` | ✅ | ✅ | ✅ | ✅ |

## movies

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /movie/{movieId} | `movie_get()` | ✅ | ✅ | ✅ | ✅ |
| GET | /movie/{movieId}/ratings | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/ratingscombined | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/recommendations | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /movie/{movieId}/similar | - | ⬜ | ⬜ | ⬜ | ⬜ |

## tv

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tv/{tvId} | `tv_get()` | ✅ | ✅ | ✅ | ✅ |
| GET | /tv/{tvId}/ratings | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/recommendations | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/season/{seasonId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tv/{tvId}/similar | - | ⬜ | ⬜ | ⬜ | ⬜ |

## other

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /keyword/{keywordId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/movies | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/regions | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /watchproviders/tv | - | ⬜ | ⬜ | ⬜ | ⬜ |

## person

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /person/{personId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /person/{personId}/combined_credits | - | ⬜ | ⬜ | ⬜ | ⬜ |

## media

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /media | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /media/{mediaId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /media/{mediaId}/watch_data | - | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /media/{mediaId}/{status} | - | ⬜ | ⬜ | ⬜ | ⬜ |

## collection

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /collection/{collectionId} | - | ⬜ | ⬜ | ⬜ | ⬜ |

## service

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /service/radarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/radarr/{radarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr/lookup/{tmdbId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /service/sonarr/{sonarrId} | - | ⬜ | ⬜ | ⬜ | ⬜ |

## tmdb

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /backdrops | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /genres/movie | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /genres/tv | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /languages | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /network/{networkId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /regions | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /studio/{studioId} | - | ⬜ | ⬜ | ⬜ | ⬜ |

## issue

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /issue | `issue_list()` | ✅ | ✅ | ✅ | ✅ |
| POST | /issue | `issue_create()` | ✅ | ✅ | ✅ | ✅ |
| GET | /issue/count | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /issue/{issueId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /issue/{issueId} | `issue_get()` | ✅ | ✅ | ✅ | ✅ |
| POST | /issue/{issueId}/comment | `issue_comment()` | ✅ | ✅ | ✅ | ✅ |
| POST | /issue/{issueId}/{status} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /issueComment/{commentId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /issueComment/{commentId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /issueComment/{commentId} | - | ⬜ | ⬜ | ⬜ | ⬜ |
