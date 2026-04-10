# ByteStash API Coverage

**Last updated:** 2026-04-09  
**Source spec:** `docs/upstream-api/bytestash.md`  
**SDK surface:** `crates/lab-apis/src/bytestash/client.rs` (18 public methods: 17 wrappers + `probe()`)  
**Shared dispatch:** `crates/lab/src/dispatch/bytestash.rs` + `crates/lab/src/dispatch/bytestash/` (catalog, client, params, dispatch)  
**MCP adapter:** `crates/lab/src/mcp/services/bytestash.rs` (thin adapter over shared dispatch)  
**CLI surface:** `crates/lab/src/cli/bytestash.rs` (generic `action` + `key=value` params → calls shared `dispatch`)  
**API handler:** `crates/lab/src/api/services/bytestash.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and live-tested |
| ⬜ | Implemented, not yet live-tested |
| ❌ | Removed — endpoint does not exist in ByteStash |
| — | Not applicable |

> **Auth note:** `ByteStash` uses a non-standard header `bytestashauth: Bearer <jwt>`.
> The client sends this via `Auth::ApiKey { header: "bytestashauth", key: "Bearer <token>" }`.
> `BYTESTASH_TOKEN` must be a valid JWT (obtained via `auth.login`), not an API key.
>
> API key auth is currently broken upstream — `authenticateToken` runs after
> `apiKeyAuth` sets `req.user` without short-circuiting, so API keys always 401.

## SDK Surface

| Method | Purpose |
|--------|---------|
| `probe()` | Health probe via `GET /api/auth/config` |
| `auth_config()` | Auth provider configuration |
| `auth_register()` | Register a new user |
| `auth_login()` | Log in and receive a JWT |
| `snippets_list()` | List the caller's snippets |
| `snippet_get(id)` | Fetch one snippet |
| `snippets_create()` | Create a snippet |
| `snippets_update(id)` | Update a snippet |
| `snippets_delete(id)` | Delete a snippet |
| `snippets_public_list()` | List public snippets |
| `snippets_public_get(id)` | Fetch one public snippet |
| `snippets_share_create(body)` | Create a share link |
| `snippets_share_get(share_id)` | Fetch a shared snippet |
| `categories_list()` | Derive unique categories from snippets list |
| `users_list()` | List users (admin routes — newer ByteStash only) |
| `users_toggle_active(id)` | Toggle user active status (admin routes) |
| `users_delete(id)` | Delete a user (admin routes) |

## Action Catalog

### Auth

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `auth.config` | `GET /api/auth/config` | ✅ | ✅ | ✅ | ✅ |
| `auth.register` | `POST /api/auth/register` | ✅ | ✅ | ⬜ | ⬜ |
| `auth.login` | `POST /api/auth/login` | ✅ | ✅ | ✅ | ✅ |
| ~~`auth.refresh`~~ | ~~`POST /api/auth/refresh`~~ | ❌ | ❌ | ❌ | ❌ |

### Snippets

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `snippets.list` | `GET /api/snippets` | ✅ | ✅ | ✅ | ✅ |
| `snippets.get` | `GET /api/snippets/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.create` | `POST /api/snippets` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.update` | `PUT /api/snippets/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.delete` | `DELETE /api/snippets/:id` | ✅ | ✅ | ⬜ | ⬜ |

### Public / Shared Snippets

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `snippets.public.list` | `GET /api/public/snippets` | ✅ | ✅ | ✅ | ✅ |
| `snippets.public.get` | `GET /api/public/snippets/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.share.create` | `POST /api/share` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.share.get` | `GET /api/share/:share_id` | ✅ | ✅ | ⬜ | ⬜ |

### Categories

| Action | Endpoint | SDK | MCP | CLI | API | Notes |
|--------|----------|-----|-----|-----|-----|-------|
| `categories.list` | derived from `GET /api/snippets` | ✅ | ✅ | ✅ | ✅ | No dedicated endpoint; deduped client-side |
| ~~`categories.create`~~ | — | ❌ | ❌ | ❌ | ❌ | Categories are implicit tags on snippets |

### Users (admin — requires newer ByteStash with `/api/admin` routes)

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `users.list` | `GET /api/admin/users` | ✅ | ✅ | ⬜ | ⬜ |
| `users.toggle-active` | `PATCH /api/admin/users/:id/toggle-active` | ✅ | ✅ | ⬜ | ⬜ |
| `users.delete` | `DELETE /api/admin/users/:id` | ✅ | ✅ | ⬜ | ⬜ |

## Notes

- `BYTESTASH_URL` must point at the service root, e.g. `https://bytestash.tootie.tv`.
- `BYTESTASH_TOKEN` must be a JWT from `auth.login` — **not** an API key.
- Auth header: `bytestashauth: Bearer <jwt>` (non-standard; not `Authorization: Bearer`).
- Admin user endpoints (`users.*`) require a ByteStash deployment with `/api/admin` routes
  (added after the version currently deployed at `bytestash.tootie.tv`).
- `categories.list` has no server-side endpoint — the client fetches all snippets and
  deduplicates the category strings locally.
