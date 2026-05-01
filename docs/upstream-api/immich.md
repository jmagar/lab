# Immich Source Contract

Retrieved: 2026-05-01

Sources:
- https://docs.immich.app/api/
- https://api.immich.app/
- https://immich.app/blog/immich-api-documentation

Immich publishes OpenAPI-backed API docs and generated SDKs. The implementation keeps a small hand-written v1 surface and must refresh/pin the exact OpenAPI artifact before expanding endpoints.

## Auth

Lab uses `IMMICH_URL` and `IMMICH_API_KEY`. The API key is sent as `x-api-key` and is secret in metadata, logs, errors, and UI.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.health` | `GET /api/server/ping` | no body | safe |
| `server.info` | `GET /api/server/about` | no body | safe |
| `server.version` | `GET /api/server/version` | no body | safe |
| `user.me` | `GET /api/users/me` | allowlisted/redacted | sensitive user summary |
| `asset.search` | `POST /api/search/metadata` | required `limit`, max 50, no auto-drain | sensitive media metadata |
| `asset.get` | `GET /api/assets/{id}` | one id | sensitive media metadata |

## Security

Sensitive fields include API keys, original paths, thumbnails, download URLs, EXIF/GPS, face/person data, owner/admin fields, checksums, and raw media bytes. v1 strips common sensitive asset fields and does not expose upload, delete, share, thumbnail, download, admin, job, or library mutation surfaces.

