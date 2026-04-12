# Linkding API Coverage

**Last updated:** 2026-04-12
**Source spec:** `docs/upstream-api/linkding.md`
**SDK surface:** `crates/lab-apis/src/linkding/client.rs` (13 public methods: 12 wrappers + `probe()`)
**Shared dispatch:** `crates/lab/src/dispatch/linkding.rs` + `crates/lab/src/dispatch/linkding/` (catalog, client, params, dispatch)
**MCP adapter:** `crates/lab/src/mcp/services/linkding.rs` (tests-only; catalog lives in dispatch layer)
**CLI surface:** `crates/lab/src/cli/linkding.rs` (generic `action` + `params` stub → calls dispatch)
**API handler:** `crates/lab/src/api/services/linkding.rs` (thin adapter over shared dispatch)

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

Live smoke tests run 2026-04-12 against `https://ding.tootie.tv`.

| Surface | Command | Result |
|---------|---------|--------|
| CLI | `lab linkding bookmarks.list` | 381 bookmarks returned |
| CLI | `lab linkding tags.list` | 299 tags returned |
| CLI | `lab linkding user.profile` | profile loaded |
| MCP | `mcporter call lab.linkding action=bookmarks.list` | `ok=true, count=381` |
| MCP | `mcporter call lab.linkding action=tags.list` | `ok=true, count=299` |
| API | `POST /v1/linkding {"action":"bookmarks.list"}` | 381 bookmarks |
| API | `POST /v1/linkding {"action":"tags.list"}` | 299 tags |

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

## Action Catalog

### Bookmarks

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `bookmarks.list` | `GET /api/bookmarks/` | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.archived.list` | `GET /api/bookmarks/archived/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.get` | `GET /api/bookmarks/:id/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.check` | `GET /api/bookmarks/check/?url=` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.create` | `POST /api/bookmarks/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.update` | `PATCH /api/bookmarks/:id/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.archive` | `POST /api/bookmarks/:id/archive/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.unarchive` | `POST /api/bookmarks/:id/unarchive/` | ✅ | ⬜ | ⬜ | ⬜ |
| `bookmarks.delete` | `DELETE /api/bookmarks/:id/` | ✅ | ⬜ | ⬜ | ⬜ |

### Tags

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `tags.list` | `GET /api/tags/` | ✅ | ✅ | ✅ | ✅ |
| `tags.get` | `GET /api/tags/:id/` | ✅ | ⬜ | ⬜ | ⬜ |
| `tags.create` | `POST /api/tags/` | ✅ | ⬜ | ⬜ | ⬜ |

### User

| Action | Endpoint | SDK | MCP | CLI | API |
|--------|----------|-----|-----|-----|-----|
| `user.profile` | `GET /api/user/profile/` | ✅ | ⬜ | ✅ | ⬜ |

## Notes

- `LINKDING_URL` must point at the service root, e.g. `http://linkding.local:9090`.
- `LINKDING_TOKEN` is the REST API token from the Linkding Settings page (not a user password).
- Auth header: `Authorization: Token <token>` (standard Django REST Framework token auth).
- `bookmarks.archive` and `bookmarks.delete` are marked `destructive: true`.
- `tags.create` is marked `destructive: true` (mutates tag namespace).
- `bookmarks.update` uses PATCH semantics — only provided fields are changed.
- `bookmarks.create` auto-scrapes title/description from the URL if not provided.
