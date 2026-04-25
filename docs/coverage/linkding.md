# Linkding API Coverage

**Last updated:** 2026-04-14
**Source spec:** `docs/upstream-api/linkding.md`
**SDK surface:** `crates/lab-apis/src/linkding/client.rs` (20 public methods: 19 wrappers + `probe()`)
**Shared dispatch:** `crates/lab/src/dispatch/linkding/` (catalog.rs, dispatch.rs, client.rs, params.rs)
**MCP registration:** `crates/lab/src/registry.rs` (direct registry entry to dispatch layer)
**CLI surface:** `crates/lab/src/cli/linkding.rs` (Tier 2: generic `action` + `--params JSON` shim)
**API handler:** `crates/lab/src/api/services/linkding.rs` (thin dispatch adapter with bearer auth + confirmation gating)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and live-tested |
| ⬜ | Implemented, not yet live-tested |
| — | Not applicable |

> **Auth note:** Linkding uses `Authorization: Token <api_token>`.
> The token is the REST API token from the Linkding Settings page.
> Env vars: `LINKDING_URL`, `LINKDING_TOKEN`.

## Live Test Evidence

Live smoke tests run 2026-04-12 against `<LINKDING_URL>`.

| Surface | Command | Result |
|---------|---------|--------|
| CLI | `lab linkding bookmarks.list` | bookmarks returned (live count) |
| CLI | `lab linkding tags.list` | tags returned (live count) |
| CLI | `lab linkding user.profile` | profile loaded |
| MCP | `mcporter call lab.linkding action=bookmarks.list` | `ok=true`, bookmarks returned |
| MCP | `mcporter call lab.linkding action=tags.list` | `ok=true`, tags returned |
| API | `POST /v1/linkding {"action":"bookmarks.list"}` | bookmarks returned |
| API | `POST /v1/linkding {"action":"tags.list"}` | tags returned |

## SDK Surface

| Method | Purpose |
|--------|---------|
| `probe()` | Health probe via `GET /api/bookmarks/?limit=1` |
| `bookmarks_list(params)` | List bookmarks with optional search/pagination |
| `bookmarks_archived_list(params)` | List archived bookmarks |
| `bookmark_get(id)` | Retrieve a single bookmark by ID |
| `bookmark_check(url)` | Check if a URL is already bookmarked |
| `bookmark_create(body)` | Create a new bookmark |
| `bookmark_update(id, body)` | Partially update a bookmark (PATCH) |
| `bookmark_archive(id)` | Archive a bookmark |
| `bookmark_unarchive(id)` | Unarchive a bookmark |
| `bookmark_delete(id)` | Delete a bookmark by ID |
| `tags_list()` | List all tags |
| `tag_get(id)` | Retrieve a single tag by ID |
| `tag_create(body)` | Create a new tag |
| `user_profile()` | Retrieve user profile / preferences |
| `bundles_list()` | List all bundles (saved searches) |
| `bundle_create(body)` | Create a new bundle |
| `bundle_update(id, patch)` | Partially update a bundle (PATCH) |
| `bundle_delete(id)` | Delete a bundle by ID |
| `bookmark_assets(id)` | List assets attached to a bookmark |
| `bookmark_assets_upload(id, name, bytes)` | Upload an asset for a bookmark (multipart) |

## Action Catalog

All 21 actions (plus 2 built-ins: `help`, `schema`) are fully implemented across SDK, dispatch, MCP, CLI, and API surfaces.

### Bookmarks

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bookmark.list` | `GET /api/bookmarks/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.archived.list` | `GET /api/bookmarks/archived/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.get` | `GET /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.check` | `GET /api/bookmarks/check/?url=` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.create` | `POST /api/bookmarks/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.update` | `PATCH /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.archive` | `POST /api/bookmarks/:id/archive/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.unarchive` | `POST /api/bookmarks/:id/unarchive/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.delete` | `DELETE /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### Tags

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `tag.list` | `GET /api/tags/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tag.get` | `GET /api/tags/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tag.create` | `POST /api/tags/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### User

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `user.profile` | `GET /api/user/profile/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### Bundles

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bundle.list` | `GET /api/bundles/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bundle.create` | `POST /api/bundles/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bundle.update` | `PATCH /api/bundles/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bundle.delete` | `DELETE /api/bundles/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### Bookmark Assets

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bookmark.assets` | `GET /api/bookmarks/:id/assets/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmark.assets-upload` | `POST /api/bookmarks/:id/assets/upload/` | ✅ | ✅ | ✅ | ✅ | ✅ |

## Action Parameters

### `bookmark.list` / `bookmark.archived.list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `q` | string | no | Search phrase |
| `limit` | integer | no | Maximum number of results (default 100) |
| `offset` | integer | no | Index from which to start returning results |

### `bookmark.get` / `bookmark.archive` / `bookmark.unarchive` / `bookmark.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |

### `bookmark.check`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `url` | string | yes | URL to check |

### `bookmark.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `url` | string | yes | Bookmark URL |
| `title` | string | no | Optional title (auto-scraped if omitted) |
| `description` | string | no | Optional description |
| `notes` | string | no | Optional markdown notes |
| `is_archived` | bool | no | Save directly to archive |
| `unread` | bool | no | Mark as unread |
| `shared` | bool | no | Share publicly |
| `tag_names` | json | no | Array of tag names to assign |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

### `bookmark.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |
| `url` | string | no | New URL |
| `title` | string | no | New title |
| `description` | string | no | New description |
| `notes` | string | no | New notes |
| `unread` | bool | no | Update unread status |
| `shared` | bool | no | Update shared status |
| `tag_names` | json | no | Replace tag names (full replacement, not append) |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

### `tag.get`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Tag ID |

### `tag.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Tag name |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

## Action Parameters

### `bundle.list`
No parameters.

### `bundle.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Bundle name |
| `search_query` | string | yes | Linkding search query that populates this bundle |
| `description` | string | no | Optional description |

### `bundle.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bundle ID |
| `payload` | json | yes | JSON object with fields to update (`name`, `search_query`, `description`) |

### `bundle.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bundle ID |

### `bookmark.assets`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |

### `bookmark.assets-upload`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |
| `file_name` | string | yes | Filename to use for the uploaded asset |
| `file_base64` | string | yes | Base64-encoded file content to upload |

## Destructive Actions

The following actions are marked `destructive: true` in the catalog:

| Action | CLI (`-y` required) | API (`"confirm": true` required) |
|--------|---------------------|----------------------------------|
| `bookmark.delete` | yes | yes |
| `bundle.delete` | yes | yes |

All other actions are reversible and do not require confirmation.

> **Note:** The CLI dispatcher uses `run_action_command()`, which enforces the `-y` / `--yes` flag
> for destructive operations on non-TTY or when called non-interactively.
> The API surface enforces `"confirm": true` in the request params JSON.

## CLI Interface

The CLI uses a generic dispatch shim (Tier 2), not typed subcommands:

```bash
lab linkding <action> [--params '<json>']

# List bookmarks
lab linkding bookmark.list
lab linkding bookmark.list --params '{"q":"rust","limit":10}'

# Create a bookmark
lab linkding bookmark.create --params '{"url":"https://example.com","title":"Example"}'

# List and get tags
lab linkding tag.list
lab linkding tag.get --params '{"id":1}'
lab linkding tag.create --params '{"name":"rust"}'

# User profile
lab linkding user.profile

# Bundles (saved searches)
lab linkding bundle.list
lab linkding bundle.create --params '{"name":"Rust reads","search_query":"#rust"}'
lab linkding bundle.update --params '{"id":1,"payload":{"name":"Updated"}}'
lab linkding bundle.delete --params '{"id":1}' -y

# Bookmark assets
lab linkding bookmark.assets --params '{"id":42}'
lab linkding bookmark.assets-upload --params '{"id":42,"file_name":"page.pdf","file_base64":"<base64>"}'

# Destructive action
lab linkding bookmark.delete --params '{"id":123}' -y
```

## Implementation Notes

### Configuration

- `LINKDING_URL` must point at the service root, e.g. `http://linkding.local:9090`.
- `LINKDING_TOKEN` is the REST API token from the Linkding Settings page (not a user password).
- Auth header: `Authorization: Token <token>` (standard Django REST Framework token auth).

### Action Semantics

- `bookmark.update` and `bundle.update` use PATCH semantics — only provided fields are changed.
- `bookmark.create` auto-scrapes title/description from the URL if not provided.
- `bookmark.unarchive` is a reversible operation (not marked destructive).
- `bundle.update` requires a `payload` JSON object with fields to update (`name`, `search_query`, `description`).

### Assets

- `bookmark.assets-upload` accepts `file_base64` (standard base64 encoding) as the file content.
- The dispatch layer decodes the base64 before constructing the multipart/form-data request to Linkding.

### Catalog

- Total: 21 service actions (plus 2 built-in actions: `help`, `schema`).
- No multi-instance support — single default instance via `LINKDING_URL` and `LINKDING_TOKEN`.
- SDK method naming uses singular form (e.g., `client.bookmark_get()`) for consistency with Rust style.
- Action naming uses singular form where appropriate (e.g., `bookmark.*`, `tag.*`, `bundle.*`).
