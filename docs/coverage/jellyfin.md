# Jellyfin Coverage

Source contract: `docs/upstream-api/jellyfin.openapi.json`

Upstream: official stable Jellyfin OpenAPI (`https://api.jellyfin.org/openapi/jellyfin-openapi-stable.json`)

- OpenAPI: `3.0.1`
- Jellyfin API version: `10.11.8`
- Path count: `315`
- Auth scheme: `CustomAuthentication` in the `Authorization` header
- Service name: `jellyfin`
- Env prefix: `JELLYFIN`
- Default port: `8096`

## First Wave Scope

The first implementation is read-heavy only. It intentionally does not expose all upstream endpoints.

| Area | Upstream path | SDK method | Dispatch action | CLI command | MCP action | API exposure | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| System | `GET /System/Ping` | `ping` | `system.ping` | `lab jellyfin system ping` | `system.ping` | `POST /v1/jellyfin` | implemented |
| System | `GET /System/Info` | `system_info` | `system.info` | `lab jellyfin system info` | `system.info` | `POST /v1/jellyfin` | implemented |
| System | `GET /System/Info/Public` | `public_system_info` | `system.public_info` | `lab jellyfin system public-info` | `system.public_info` | `POST /v1/jellyfin` | implemented |
| Users | `GET /Users` | `users` | `users.list` | `lab jellyfin users list` | `users.list` | `POST /v1/jellyfin` | implemented |
| Users | `GET /Users/Me` | `current_user` | `users.me` | `lab jellyfin users me` | `users.me` | `POST /v1/jellyfin` | implemented |
| Libraries | `GET /Library/VirtualFolders` | `libraries` | `libraries.list` | `lab jellyfin libraries list` | `libraries.list` | `POST /v1/jellyfin` | implemented |
| Items | `GET /Items` | `items` | `items.search` | `lab jellyfin items search` | `items.search` | `POST /v1/jellyfin` | implemented |
| Items | `GET /Items/{itemId}` | `item` | `items.get` | `lab jellyfin items get` | `items.get` | `POST /v1/jellyfin` | implemented |
| Items | `GET /Items/Counts` | `item_counts` | `items.counts` | `lab jellyfin items counts` | `items.counts` | `POST /v1/jellyfin` | implemented |
| Sessions | `GET /Sessions` | `sessions` | `sessions.list` | `lab jellyfin sessions list` | `sessions.list` | `POST /v1/jellyfin` | implemented |
| Plugins | `GET /Plugins` | `plugins` | `plugins.list` | `lab jellyfin plugins list` | `plugins.list` | `POST /v1/jellyfin` | implemented |

## Bounded Item Search Params

`items.search` starts with an explicit subset of safe query params:

- `user_id`
- `search_term`
- `parent_id`
- `include_item_types`
- `recursive`
- `start_index`
- `limit`

Arbitrary upstream query parameters are deferred so callers cannot accidentally route into sensitive or expensive search shapes.

## Deferred Surfaces

These upstream areas are intentionally deferred from the first wave:

- Login/password authentication endpoints such as `POST /Users/AuthenticateByName`.
- Media download/file endpoints such as `GET /Items/{itemId}/Download` and `GET /Items/{itemId}/File`.
- Library mutations such as virtual folder create/update/delete and `POST /Library/Refresh`.
- Item mutations such as metadata refresh, image mutation, and delete operations.
- Session control endpoints such as playback commands, messages, logout, and user transfer.
- Plugin mutation endpoints such as enable, disable, install, uninstall, and configuration writes.
- User creation, password, policy, and configuration mutation.

When these are introduced, each action must be reviewed for `ActionSpec.destructive = true` and confirmation behavior across CLI, MCP, and API.

## Live Test Evidence

Status: pending.

No live Jellyfin instance was available during implementation in this workspace. Use non-destructive smoke tests when credentials are available:

```bash
JELLYFIN_URL=http://localhost:8096 JELLYFIN_API_KEY=... cargo run --all-features -- jellyfin system info --json
mcporter call jellyfin '{"action":"system.info","params":{}}'
curl -sS -X POST http://127.0.0.1:3000/v1/jellyfin -H 'content-type: application/json' -d '{"action":"system.info","params":{}}'
```

Observability proof must include one successful Jellyfin action and one failing action with no `Authorization`, API key, or raw secret env value in logs.
