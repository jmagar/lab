# Gotify API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** `docs/upstream-api/gotify.openapi.json`
**API version:** 2.0.2
**SDK surface:** `crates/lab-apis/src/gotify/client.rs` (26 public methods)
**Shared dispatch:** `crates/lab/src/dispatch/gotify.rs` + `crates/lab/src/dispatch/gotify/` (catalog, client, params, dispatch)
**MCP adapter:** `crates/lab/src/mcp/services/gotify.rs` (thin re-export of dispatch layer)
**CLI surface:** `crates/lab/src/cli/gotify.rs` (generic `action` + `key=value` params → shared dispatch)
**API handler:** `crates/lab/src/api/services/gotify.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and live-tested |
| ⬜ | Implemented, not yet live-tested |
| — | Not implemented (out of scope for initial cut) |

> **Auth note:** Gotify uses the `X-Gotify-Key` header.
> App tokens have send-only scope; client tokens have read/manage scope.
>
> Env vars:
> - `GOTIFY_URL` — base URL, e.g. `http://gotify.home:8080`
> - `GOTIFY_APP_TOKEN` — app token (for `message.send`)
> - `GOTIFY_CLIENT_TOKEN` — client token (for all management operations)
> - `GOTIFY_TOKEN` — legacy fallback: used as both app and client token when scoped vars are absent
>
> `server.health` requires no authentication.

## Implemented Actions

### Messages

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `message.send` | `POST /message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `message.list` | `GET /message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `message.delete` | `DELETE /message/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `message.purge` | `DELETE /message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Applications

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `app.list` | `GET /application` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `app.create` | `POST /application` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `app.delete` | `DELETE /application/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `application.update` | `PUT /application/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `application.messages` | `GET /application/{id}/message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `application.messages-delete` | `DELETE /application/{id}/message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Clients

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `client.list` | `GET /client` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `client.create` | `POST /client` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `client.delete` | `DELETE /client/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `client.update` | `PUT /client/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Plugins

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `plugin.list` | `GET /plugin` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `plugin.enable` | `POST /plugin/{id}/enable` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `plugin.disable` | `POST /plugin/{id}/disable` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `plugin.config-get` | `GET /plugin/{id}/config` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `plugin.config-set` | `POST /plugin/{id}/config` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Users

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `user.list` | `GET /user` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `user.create` | `POST /user` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `user.delete` | `DELETE /user/{id}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Server

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `server.health` | `GET /health` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `server.version` | `GET /version` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

## Action Parameters

### `message.send`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `message` | string | yes | Notification body text |
| `title` | string | no | Notification title |
| `priority` | integer | no | 0–10; validated at dispatch; higher values bypass Do Not Disturb |

### `message.list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `limit` | integer | no | Max messages to return (default 100, max 200 enforced by dispatch) |

### `message.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Message id |

### `message.purge`
No params. Destructive — deletes all messages on the server.

### `app.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Application name |
| `description` | string | no | Optional description |

### `app.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Application id |

### `application.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Application id |
| `name` | string | yes | New application name |
| `description` | string | no | Optional description |

### `application.messages`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Application id |
| `limit` | integer | no | Max messages to return |

### `application.messages-delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Application id |
Destructive — deletes ALL messages for the application.

### `client.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Client name |

### `client.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Client id |

### `client.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Client id |
| `name` | string | yes | New client name |

### `plugin.enable` / `plugin.disable`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Plugin id |

### `plugin.config-get`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Plugin id |
Returns `{config: "<yaml text>"}`.

### `plugin.config-set`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Plugin id |
| `config` | string | yes | YAML configuration string |

### `user.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Username |
| `pass` | string | yes | Password |
| `admin` | bool | no | Whether the user is an admin (default: false) |

### `user.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | User id |

### `server.version`
No params. Returns `{version, commit, buildDate}`.

## Destructive Actions

The following actions are marked `destructive: true` in the catalog:

| Action | Confirmation (CLI) | Confirmation (API) |
|--------|-------------------|--------------------|
| `message.delete` | `-y` / `--yes` flag | `"confirm": true` in params |
| `message.purge` | `-y` / `--yes` flag | `"confirm": true` in params |
| `app.delete` | `-y` / `--yes` flag | `"confirm": true` in params |
| `client.delete` | `-y` / `--yes` flag | `"confirm": true` in params |
| `application.messages-delete` | `-y` / `--yes` flag | `"confirm": true` in params |
| `user.delete` | `-y` / `--yes` flag | `"confirm": true` in params |

## Spec Endpoints Not Implemented

These endpoints exist in the OpenAPI spec but remain out of scope:

| Method | Endpoint | Notes |
|--------|----------|-------|
| POST | /application/{id}/image | Image upload; low priority |
| DELETE | /application/{id}/image | Image management; low priority |
| GET | /current/user | Current user info; add if needed |
| POST | /current/user/password | Password change; low priority |
| GET | /plugin/{id}/display | Plugin display; low priority |
| GET | /stream | WebSocket stream; not applicable to HTTP/MCP action model |
| PUT | /user/{id} | Update user; add if needed |
