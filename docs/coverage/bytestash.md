# ByteStash API Coverage

**Last updated:** 2026-04-13
**Source spec:** `docs/upstream-api/bytestash.md`
**SDK surface:** `crates/lab-apis/src/bytestash/client.rs`
**Shared dispatch:** `crates/lab/src/dispatch/bytestash/` (catalog.rs, client.rs, params.rs, dispatch.rs)
**MCP adapter:** `crates/lab/src/mcp/services/bytestash.rs` (tests only; registry wires dispatch directly)
**CLI surface:** `crates/lab/src/cli/bytestash.rs` (generic `action` + `key=value` params → `run_confirmable_action_command`)
**API handler:** `crates/lab/src/api/services/bytestash.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented |
| ⬜ | Not implemented |
| ❌ | Removed — endpoint does not exist in ByteStash |
| — | Not applicable |

> **Auth note:** ByteStash uses a non-standard header `bytestashauth: Bearer <jwt>`.
> The client sends this via `Auth::ApiKey { header: "bytestashauth", key: "Bearer <token>" }`.
> `BYTESTASH_TOKEN` must be a valid JWT (obtained via `auth.login`), not a raw API key.
>
> API key auth is currently broken upstream — `authenticateToken` runs after
> `apiKeyAuth` sets `req.user` without short-circuiting, so API keys always 401.

## Config

- `BYTESTASH_URL` — base URL, e.g. `https://bytestash.tootie.tv` (required)
- `BYTESTASH_TOKEN` — JWT from `auth.login` (required; not an API key)

## SDK Surface (`lab-apis`)

| Method | Endpoint | Notes |
|--------|----------|-------|
| `probe()` | `GET /api/auth/config` | Health probe; returns `()` |
| `auth_config()` | `GET /api/auth/config` | Auth provider configuration |
| `auth_register(body)` | `POST /api/auth/register` | Register a new user |
| `auth_login(body)` | `POST /api/auth/login` | Log in and receive a JWT |
| `snippets_list()` | `GET /api/snippets` | List the caller's snippets |
| `snippet_get(id)` | `GET /api/snippets/:id` | Fetch one snippet |
| `snippets_create(body)` | `POST /api/snippets` | Create a snippet |
| `snippets_update(id, body)` | `PUT /api/snippets/:id` | Update a snippet |
| `snippets_delete(id)` | `DELETE /api/snippets/:id` | Delete a snippet |
| `snippets_public_list()` | `GET /api/public/snippets` | List public snippets |
| `snippets_public_get(id)` | `GET /api/public/snippets/:id` | Fetch one public snippet |
| `snippets_share_create(body)` | `POST /api/share` | Create a share link |
| `snippets_share_get(share_id)` | `GET /api/share/:share_id` | Fetch a shared snippet |
| `categories_list()` | derived from `GET /api/snippets` | Deduped categories; no dedicated endpoint |
| `users_list()` | `GET /api/admin/users` | List users (admin routes — newer ByteStash only) |
| `users_toggle_active(id)` | `PATCH /api/admin/users/:id/toggle-active` | Toggle user active status (admin only) |
| `users_delete(id)` | `DELETE /api/admin/users/:id` | Delete a user (admin only) |

## Action Catalog

All actions are dispatched via the shared dispatch layer in `crates/lab/src/dispatch/bytestash/`.
The catalog is the single source of truth in `catalog.rs`.

The CLI uses `run_confirmable_action_command` — all actions in the catalog are accessible
via `lab bytestash <action> [key=value ...]`. Destructive actions require `-y` / `--yes` or
`--dry-run`. There is no separate whitelist for "CLI-only" actions.

### Auth

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `auth.config` | `GET /api/auth/config` | ✅ | no | ✅ | ✅ | ✅ |
| `auth.register` | `POST /api/auth/register` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `auth.login` | `POST /api/auth/login` | ✅ | no | ✅ | ✅ | ✅ |
| ~~`auth.refresh`~~ | ~~`POST /api/auth/refresh`~~ | ❌ | — | ❌ | ❌ | ❌ |

### Snippets

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `snippets.list` | `GET /api/snippets` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.get` | `GET /api/snippets/:id` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.create` | `POST /api/snippets` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.update` | `PUT /api/snippets/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.delete` | `DELETE /api/snippets/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |

### Public / Shared Snippets

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `snippets.public.list` | `GET /api/public/snippets` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.public.get` | `GET /api/public/snippets/:id` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.share.create` | `POST /api/share` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.share.get` | `GET /api/share/:share_id` | ✅ | no | ✅ | ✅ | ✅ |

### Categories

| Action | Endpoint | SDK | Destructive | MCP | CLI | API | Notes |
|--------|----------|-----|-------------|-----|-----|-----|-------|
| `categories.list` | derived from `GET /api/snippets` | ✅ | no | ✅ | ✅ | ✅ | No dedicated endpoint; deduped client-side |
| ~~`categories.create`~~ | — | ❌ | — | ❌ | ❌ | ❌ | Categories are implicit tags on snippets |

### Users (admin — requires newer ByteStash with `/api/admin` routes)

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `users.list` | `GET /api/admin/users` | ✅ | no | ✅ | ✅ | ✅ |
| `users.toggle-active` | `PATCH /api/admin/users/:id/toggle-active` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `users.delete` | `DELETE /api/admin/users/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |

## Key Param Details

**`auth.register` and `auth.login`** accept `username` + `password` as individual params, or a
`payload` JSON object as a full body override.

**`snippets.create` and `snippets.update`** accept `title` (required for create), `description`,
`language`, `fragments` (JSON), `categories` (JSON), or a `payload` JSON override. `snippets.update`
also requires `id`.

**`snippets.share.create`** accepts `snippetId` (required), `requiresAuth` (bool), `expiresIn`
(integer seconds; null = never), or a `payload` JSON override.

**`snippets.share.get`** requires `share_id`.

**`users.toggle-active` and `users.delete`** require `id`.

## Surface Details

### CLI (`lab bytestash`)

Tier 2 (dispatch-backed thin shim). Accepts positional `action` and trailing `key=value` params.
Supports `-y` / `--yes` (alias `--no-confirm`) and `--dry-run`.

```
lab bytestash help
lab bytestash snippets.list
lab bytestash snippets.get id=abc123
lab bytestash snippets.create title="My snippet" language=rust
lab bytestash snippets.delete id=abc123 -y
lab bytestash auth.login username=admin password=secret
```

### MCP

Single tool `bytestash`. The `mcp/services/bytestash.rs` file contains only tests; the registry
in `mcp/registry.rs` wires `crate::dispatch::bytestash::dispatch` directly.

### API (`POST /v1/bytestash`)

Standard action+params envelope. Uses `AppState.clients.bytestash` (pre-built at startup from
`BYTESTASH_URL` + `BYTESTASH_TOKEN`). Destructive actions require `"confirm": true` in params.

## Notes

- `BYTESTASH_URL` must point at the service root, e.g. `https://bytestash.tootie.tv`.
- `BYTESTASH_TOKEN` must be a JWT from `auth.login` — **not** a static API key.
- Auth header: `bytestashauth: Bearer <jwt>` (non-standard; not `Authorization: Bearer`).
- Admin user endpoints (`users.*`) require a ByteStash deployment with `/api/admin` routes
  (added after the version currently deployed at `bytestash.tootie.tv`).
- `categories.list` has no server-side endpoint — the client fetches all snippets and
  deduplicates the category strings locally.
- `dispatch/bytestash/client.rs` also exposes `client_from_vars(url, token)` for callers
  that supply explicit values rather than reading env vars.
