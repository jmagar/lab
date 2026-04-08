# ByteStash API Coverage

**Last updated:** 2026-04-08  
**Source spec:** `docs/upstream-api/bytestash.md`  
**SDK surface:** `crates/lab-apis/src/bytestash/client.rs` (19 public methods: 18 wrappers + `probe()`)  
**MCP actions:** `crates/lab/src/mcp/services/bytestash.rs` (18 actions + built-in `help`)  
**CLI surface:** `crates/lab/src/cli/bytestash.rs` (generic `action` + `key=value` params)  
**HTTP API handler:** `crates/lab/src/api/services/bytestash.rs` (same dispatch contract as MCP)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in code |
| ⬜ | Not live-tested in this workspace |
| — | Not applicable |

> ByteStash is now wired as a thin action dispatcher across SDK, CLI, MCP, and HTTP.
> The documented REST surface is implemented, but destructive endpoints were not
> exercised against a live ByteStash instance. Read-only smoke checks were run
> against a local mock controller for `auth.config`, `snippets.list`, and
> `categories.list` through both CLI and MCP.

## SDK Surface

| Method | Purpose |
|--------|---------|
| `probe()` | Health probe via `/api/auth/config` |
| `auth_config()` | Auth provider configuration |
| `auth_register()` | Register a new user |
| `auth_login()` | Log in and receive a JWT |
| `auth_refresh()` | Refresh the current JWT |
| `snippets_list()` | List the caller's snippets |
| `snippet_get(id)` | Fetch one snippet |
| `snippets_create()` | Create a snippet |
| `snippets_update(id)` | Update a snippet |
| `snippets_delete(id)` | Delete a snippet |
| `snippets_public_list()` | List public snippets |
| `snippets_public_get(id)` | Fetch one public snippet |
| `snippets_share_create(id)` | Create a share link |
| `snippets_share_get(share_id)` | Fetch a shared snippet |
| `categories_list()` | List categories |
| `categories_create()` | Create a category |
| `users_list()` | List users |
| `users_patch(id)` | Patch a user |
| `users_delete(id)` | Delete a user |

## Action Catalog

### Auth

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `auth.config` | `GET /api/auth/config` | ✅ | ✅ | ✅ | ✅ |
| `auth.register` | `POST /api/auth/register` | ✅ | ✅ | ✅ | ✅ |
| `auth.login` | `POST /api/auth/login` | ✅ | ✅ | ✅ | ✅ |
| `auth.refresh` | `POST /api/auth/refresh` | ✅ | ✅ | ✅ | ✅ |

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
| `snippets.public.list` | `GET /api/snippets/public` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.public.get` | `GET /api/snippets/public/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.share.create` | `POST /api/snippets/share/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `snippets.share.get` | `GET /api/snippets/share/:share_id` | ✅ | ✅ | ⬜ | ⬜ |

### Categories

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `categories.list` | `GET /api/categories` | ✅ | ✅ | ✅ | ✅ |
| `categories.create` | `POST /api/categories` | ✅ | ✅ | ⬜ | ⬜ |

### Users

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `users.list` | `GET /api/users` | ✅ | ✅ | ⬜ | ⬜ |
| `users.patch` | `PATCH /api/users/:id` | ✅ | ✅ | ⬜ | ⬜ |
| `users.delete` | `DELETE /api/users/:id` | ✅ | ✅ | ⬜ | ⬜ |

## Notes

- `BYTESTASH_URL` must point at the service root, for example `http://localhost:5000`.
- `BYTESTASH_TOKEN` is passed as `Authorization: Bearer <token>`.
- The client uses `/api` paths directly; callers do not need to add the prefix.
- CLI and MCP use the same action catalog and param parsing, so the documented
  `action` names are the single contract for both surfaces.
