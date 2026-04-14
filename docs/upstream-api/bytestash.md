# ByteStash API Reference

> **No static OpenAPI spec is committed to the repo.** The spec is generated at runtime by the Express backend (likely via `swagger-jsdoc`) and served at `/api-docs` on every running instance.
>
> Source: https://github.com/jordan-dalby/ByteStash

## How to fetch the spec from your own instance

```bash
# Swagger UI (HTML)
open "$BYTESTASH_URL/api-docs"

# Raw JSON (path may vary — common defaults)
curl -fsSL "$BYTESTASH_URL/api-docs/swagger.json" -o docs/upstream-api/bytestash.openapi.json
# or
curl -fsSL "$BYTESTASH_URL/api-docs.json" -o docs/upstream-api/bytestash.openapi.json
```

If neither path works, inspect the network requests in the running Swagger UI to find the actual JSON endpoint.

## Quick reference (high-level endpoint groups)

> Source: ByteStash README + source inspection.

REST, JWT bearer auth (`Authorization: Bearer <token>`). Base URL: `$BYTESTASH_URL/api/`.

### Authentication
- `POST   /api/auth/register` — `{ username, password }`
- `POST   /api/auth/login` — returns `{ token }`
- `POST   /api/auth/refresh` — refresh JWT
- `GET    /api/auth/config` — auth provider config

### Snippets
- `GET    /api/snippets` — list user's snippets
- `GET    /api/snippets/:id` — single snippet
- `POST   /api/snippets` — create `{ title, description, language, fragments: [{ file_name, code }], categories }`
- `PUT    /api/snippets/:id` — update
- `DELETE /api/snippets/:id`

### Public/Shared Snippets
- `GET    /api/snippets/public` — list public snippets
- `GET    /api/snippets/public/:id` — view public snippet
- `POST   /api/snippets/share/:id` — create share link
- `GET    /api/snippets/share/:share_id` — view shared snippet

### Categories
- `GET    /api/categories` — list categories
- `POST   /api/categories` — create

### Users (admin)
- `GET    /api/users`
- `PATCH  /api/users/:id`
- `DELETE /api/users/:id`

## Implementation note for `lab-apis`

Once we have the live spec from a running instance, save it as `bytestash.openapi.json` next to this file. The endpoints above are a starting point — confirm against `/api-docs` before implementing.
