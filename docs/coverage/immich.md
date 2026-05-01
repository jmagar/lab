# Immich Coverage

Status: first-class v1 read-only metadata surface.

Actions: `server.health`, `server.info`, `server.version`, `user.me`, `asset.search`, `asset.get`, plus built-in `help` and `schema`.

Deferred: uploads, deletes, trash/archive/restore, thumbnails, downloads, sharing, albums, libraries, jobs, EXIF/GPS/person/face/admin surfaces.

Security: `IMMICH_API_KEY` is secret. Asset responses are redacted for common sensitive media fields and search requires `limit` with max 50.

