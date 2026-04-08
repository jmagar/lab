# API Specs

Reference documentation for every service `lab-apis` wraps. Used by humans and agents when implementing or auditing each service module.

## Index

| Service | File | Format | Source |
|---------|------|--------|--------|
| Radarr | `radarr.openapi.json` | OpenAPI 3.0 | [Radarr/Radarr develop branch](https://github.com/Radarr/Radarr/blob/develop/src/Radarr.Api.V3/openapi.json) |
| Sonarr | `sonarr.openapi.json` | OpenAPI 3.0 | [Sonarr/Sonarr develop branch](https://github.com/Sonarr/Sonarr/blob/develop/src/Sonarr.Api.V3/openapi.json) |
| Prowlarr | `prowlarr.openapi.json` | OpenAPI 3.0 | [Prowlarr/Prowlarr develop branch](https://github.com/Prowlarr/Prowlarr/blob/develop/src/Prowlarr.Api.V1/openapi.json) |
| Plex | `plex.openapi.yaml` | OpenAPI 3.x | [LukeHagar/plex-api-spec](https://github.com/LukeHagar/plex-api-spec) |
| Tailscale | `tailscale.openapi.yaml` | OpenAPI 3.x | `https://api.tailscale.com/api/v2?outputOpenapiSchema=true` |
| Memos | `memos.openapi.yaml` | OpenAPI from protobuf | [usememos/memos](https://github.com/usememos/memos/blob/main/proto/gen/openapi.yaml) |
| Overseerr | `overseerr.openapi.yaml` | OpenAPI 3.0.2 | [sct/overseerr](https://github.com/sct/overseerr/blob/develop/openapi.yaml) |
| Tautulli | `tautulli.md` | hand-scraped wiki | [Tautulli wiki](https://github.com/Tautulli/Tautulli/wiki/Tautulli-API-Reference) |
| SABnzbd | `sabnzbd.md` | hand-scraped wiki | [sabnzbd.org wiki](https://sabnzbd.org/wiki/configuration/4.5/api) |
| qBittorrent | `qbittorrent.md` | hand-scraped wiki | [qbit wiki](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)) + [qbittorrent-api docs](https://qbittorrent-api.readthedocs.io/en/latest/api.html) |
| Linkding | `linkding.md` | hand-scraped | [linkding.link/api](https://linkding.link/api/) |
| Paperless-ngx | `paperless.md` | stub — fetch from your instance | served at `$PAPERLESS_URL/api/schema/` |
| ByteStash | `bytestash.md` | stub — fetch from your instance | served at `$BYTESTASH_URL/api-docs` |
| Arcane | `arcane-api.yaml` | OpenAPI 3.x | [getarcane.app](https://getarcane.app) |
| Unraid | `unraid-api-complete-reference.md` | hand-compiled GraphQL reference | [docs.unraid.net/API](https://docs.unraid.net/API/) |
| UniFi | `unifi.md` | hand-scraped Network Application docs | UniFi Network controller `/proxy/network/integration/v1/` |
| Gotify | `gotify.openapi.json` | Swagger 2.0 | [gotify/server](https://github.com/gotify/server/blob/master/docs/spec.json) |
| OpenAI | `openai.openapi.yaml` | OpenAPI 3.x | [openai/openai-openapi](https://github.com/openai/openai-openapi) (`manual_spec` branch) |
| Qdrant | `qdrant.openapi.json` | OpenAPI 3.x | [qdrant/qdrant](https://github.com/qdrant/qdrant/blob/master/docs/redoc/master/openapi.json) |
| HF TEI | `tei.openapi.json` | OpenAPI 3.x | [huggingface/text-embeddings-inference](https://github.com/huggingface/text-embeddings-inference/blob/main/docs/openapi.json) |
| Apprise | `apprise.md` | hand-scraped README | [caronc/apprise-api](https://github.com/caronc/apprise-api) |

## Refreshing specs

OpenAPI specs go stale. To pull the latest:

```bash
# Radarr / Sonarr / Prowlarr (dev branch — most current)
curl -fsSL https://raw.githubusercontent.com/Radarr/Radarr/develop/src/Radarr.Api.V3/openapi.json > docs/api-specs/radarr.openapi.json
curl -fsSL https://raw.githubusercontent.com/Sonarr/Sonarr/develop/src/Sonarr.Api.V3/openapi.json > docs/api-specs/sonarr.openapi.json
curl -fsSL https://raw.githubusercontent.com/Prowlarr/Prowlarr/develop/src/Prowlarr.Api.V1/openapi.json > docs/api-specs/prowlarr.openapi.json

# Plex (community spec)
curl -fsSL https://raw.githubusercontent.com/LukeHagar/plex-api-spec/main/plex-api-spec.yaml > docs/api-specs/plex.openapi.yaml

# Tailscale
curl -fsSL "https://api.tailscale.com/api/v2?outputOpenapiSchema=true" > docs/api-specs/tailscale.openapi.yaml

# Memos
curl -fsSL https://raw.githubusercontent.com/usememos/memos/main/proto/gen/openapi.yaml > docs/api-specs/memos.openapi.yaml

# Overseerr
curl -fsSL https://raw.githubusercontent.com/sct/overseerr/develop/openapi.yaml > docs/api-specs/overseerr.openapi.yaml

# Paperless / ByteStash — fetch from YOUR instance, see paperless.md / bytestash.md

# Gotify
curl -fsSL https://raw.githubusercontent.com/gotify/server/master/docs/spec.json > docs/api-specs/gotify.openapi.json

# OpenAI (note: manual_spec branch, not master)
curl -fsSL https://raw.githubusercontent.com/openai/openai-openapi/manual_spec/openapi.yaml > docs/api-specs/openai.openapi.yaml

# Qdrant
curl -fsSL https://raw.githubusercontent.com/qdrant/qdrant/master/docs/redoc/master/openapi.json > docs/api-specs/qdrant.openapi.json

# HF Text Embeddings Inference
curl -fsSL https://raw.githubusercontent.com/huggingface/text-embeddings-inference/main/docs/openapi.json > docs/api-specs/tei.openapi.json

# Apprise (no OpenAPI spec — hand-scraped from README)
curl -fsSL https://raw.githubusercontent.com/caronc/apprise-api/master/README.md > docs/api-specs/apprise.md
```

A `just refresh-specs` target should be added that runs all of the above.

## Spec sizes (current)

```
plex.openapi.yaml         653K
qdrant.openapi.json       508K
radarr.openapi.json       302K
sonarr.openapi.json       290K
tailscale.openapi.yaml    221K
prowlarr.openapi.json     142K
memos.openapi.yaml        135K
openai.openapi.yaml       1.3M
gotify.openapi.json        67K
tei.openapi.json           51K
apprise.md                  ~K   hand-scraped README
linkding.md                 9K
qbittorrent.md              7K
tautulli.md                 7K
sabnzbd.md                  4K
paperless.md                stub
bytestash.md                stub
```
