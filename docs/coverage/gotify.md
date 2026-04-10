# Gotify API Coverage

**Last updated:** 2026-04-10  
**OpenAPI spec:** `docs/upstream-api/gotify.openapi.json`  
**API version:** 2.0.2  
**SDK surface:** `crates/lab-apis/src/gotify/client.rs` (15 public methods)  
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

> **Auth note:** Gotify uses `X-Gotify-Key` header.
> App tokens have send-only scope; client tokens have read/manage scope.
> Set `GOTIFY_TOKEN` to an app token for `message.send`, or a client token for management.

## Implemented Actions

### Messages

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `message.send` | `POST /message` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `message.list` | `GET /message` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `message.delete` | `DELETE /message/{id}` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `message.purge` | `DELETE /message` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |

### Applications

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `app.list` | `GET /application` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `app.create` | `POST /application` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `app.delete` | `DELETE /application/{id}` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |

### Clients

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `client.list` | `GET /client` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `client.create` | `POST /client` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `client.delete` | `DELETE /client/{id}` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |

### Server

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `server.health` | `GET /health` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

## Spec Endpoints Not Implemented

These endpoints exist in the OpenAPI spec but are out of scope for the initial cut:

| Method | Endpoint | Notes |
|--------|----------|-------|
| PUT | /application/{id} | `app_update()` in SDK; add `app.update` action if needed |
| POST | /application/{id}/image | Image upload; low priority |
| DELETE | /application/{id}/image | Image management; low priority |
| GET | /application/{id}/message | App-scoped read; `message.list` covers most use cases |
| DELETE | /application/{id}/message | App-scoped purge; add `app.messages.purge` if needed |
| PUT | /client/{id} | `client_update()` in SDK; add `client.update` action if needed |
| GET | /current/user | User info; add if needed |
| POST | /current/user/password | Password change; low priority |
| GET | /health | Used by `ServiceClient::health()` only (not exposed as an action) |
| GET | /plugin | Plugin management; low priority |
| GET/POST | /plugin/{id}/config | Plugin config; low priority |
| POST | /plugin/{id}/disable | Plugin management; low priority |
| GET | /plugin/{id}/display | Plugin display; low priority |
| POST | /plugin/{id}/enable | Plugin management; low priority |
| GET | /stream | WebSocket stream; not applicable to HTTP/MCP action model |
| GET/POST/DELETE | /user, /user/{id} | Admin user management; add if needed |
| GET | /version | Version info; lower priority than `server.health` |

## Notes

- `GOTIFY_URL` — base URL, e.g. `http://gotify.home:8080`
- `GOTIFY_TOKEN` — app token (send operations) or client token (management)
- `server.health` requires no authentication
- Destructive actions (`message.delete`, `message.purge`, `app.delete`, `client.delete`) require `X-Lab-Confirm: yes` header or `"confirm": true` in params
