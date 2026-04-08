# servarr/ — Shared *arr primitives

Radarr, Sonarr, and Prowlarr share ~80% of their API surface: same auth scheme, same pagination shape, same `config.xml` layout, same command/queue/history patterns. This directory holds the shared bits so we write them once.

## Rules

- **Auth header is `X-Api-Key`** (not `Authorization`). Use `Auth::ApiKey { header: "X-Api-Key".into(), value: key }`.
- **Base path is `/api/v3`** for Radarr/Sonarr, `/api/v1` for Prowlarr. Service clients own the version segment; servarr helpers should take a pre-composed base.
- **`config.xml` parser is shared** — `parse_servarr_config_xml()` lives here and is reused by `extract` for credential discovery. Do not duplicate in individual service crates.
- **Pagination shape** (`{ page, pageSize, totalRecords, sortKey, sortDirection, records }`) is shared. Define the generic `PagedResponse<T>` here, reuse in all three services.
- **Command queue polling** — `POST /command` → `{ id }`, then poll `GET /command/{id}` until `status == "completed"`. The shared helper handles backoff and cancellation.
- **Version checks** via `GET /system/status` return a `Version` string; `ServiceStatus::version` is populated from this.

## Pitfalls

- Radarr and Sonarr V3 diverged slightly in episode/movie field naming. Keep shared types to truly shared shapes; service-specific types stay in `radarr/types.rs` / `sonarr/types.rs`.
- Prowlarr's indexer test endpoint is **destructive-ish** (it hits the real tracker). Mark the action `destructive: true` or document why not.
- `config.xml` can have `UrlBase` set — the extracted URL must honor it or the scanned creds won't work.
