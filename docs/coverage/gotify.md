# Gotify API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** `docs/upstream-api/gotify.openapi.json`
**API version:** 2.0.2
**SDK surface:** `crates/lab-apis/src/gotify/client.rs` (26 public methods)
**Shared dispatch:** `crates/lab/src/dispatch/gotify/` (catalog.rs, client.rs, params.rs, dispatch.rs)
**MCP adapter:** `crates/lab/src/mcp/services/gotify.rs` (thin bridge to dispatch)
**CLI surface:** `crates/lab/src/cli/gotify.rs` (generic `action` + `key=value` params via `run_confirmable_action_command`)
**API handler:** `crates/lab/src/api/services/gotify.rs` (thin adapter via `handle_action` helper)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented across all surfaces (SDK, Dispatch, MCP, CLI, API) |
| — | Not implemented (out of scope) |

All implemented actions are live in all surfaces (MCP, CLI, API). The dispatch layer enforces destructive confirmation via `-y` flag (CLI) or `"confirm": true` param (API/MCP).

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

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `message.send` | `POST /message` | No | ✅ |
| `message.list` | `GET /message` | No | ✅ |
| `message.delete` | `DELETE /message/{id}` | Yes | ✅ |
| `message.purge` | `DELETE /message` | Yes | ✅ |

### Applications

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `app.list` | `GET /application` | No | ✅ |
| `app.create` | `POST /application` | No | ✅ |
| `app.delete` | `DELETE /application/{id}` | Yes | ✅ |
| `application.update` | `PUT /application/{id}` | No | ✅ |
| `application.messages` | `GET /application/{id}/message` | No | ✅ |
| `application.messages-delete` | `DELETE /application/{id}/message` | Yes | ✅ |

### Clients

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `client.list` | `GET /client` | No | ✅ |
| `client.create` | `POST /client` | No | ✅ |
| `client.delete` | `DELETE /client/{id}` | Yes | ✅ |
| `client.update` | `PUT /client/{id}` | No | ✅ |

### Plugins

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `plugin.list` | `GET /plugin` | No | ✅ |
| `plugin.enable` | `POST /plugin/{id}/enable` | No | ✅ |
| `plugin.disable` | `POST /plugin/{id}/disable` | No | ✅ |
| `plugin.config-get` | `GET /plugin/{id}/config` | No | ✅ |
| `plugin.config-set` | `POST /plugin/{id}/config` | No | ✅ |

### Users

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `user.list` | `GET /user` | No | ✅ |
| `user.create` | `POST /user` | No | ✅ |
| `user.delete` | `DELETE /user/{id}` | Yes | ✅ |

### Server

| Action | Endpoint | Destructive | Status |
|--------|----------|-------------|--------|
| `server.health` | `GET /health` | No | ✅ |
| `server.version` | `GET /version` | No | ✅ |

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

The following actions are marked `destructive: true` in the catalog. All require confirmation before execution:

| Action | CLI Confirmation | API Confirmation | MCP Confirmation |
|--------|------------------|------------------|------------------|
| `message.delete` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |
| `message.purge` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |
| `app.delete` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |
| `application.messages-delete` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |
| `client.delete` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |
| `user.delete` | `-y` / `--yes` flag | `"confirm": true` in params | Elicitation prompt to user |

All destructive actions are gated at the shared dispatch layer via `ActionSpec.destructive` metadata in `catalog.rs`.

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
