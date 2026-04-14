# Arcane API Coverage

**Last updated:** 2026-04-13 (added project, volume, image actions)
**OpenAPI spec:** docs/api-specs/arcane-api.yaml
**OpenAPI version:** 3.1.0
**API version:** v1.17.1
**Paths:** 265
**Servers:** https://arcane.tootie.tv/api
**Security schemes:** ApiKeyAuth, BearerAuth

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet; rows are spec inventory only |
| — | Not applicable / not represented in the spec |

The source spec is the contract. This document reflects what is actually implemented.

## Config

- `ARCANE_URL` — base URL (required)
- `ARCANE_API_KEY` — API key (required); sent as `X-API-Key` header

## SDK Surface (`lab-apis`)

All methods live in `crates/lab-apis/src/arcane/client.rs`.

| Method | Endpoint | Returns |
|--------|----------|---------|
| `health()` | `GET /health` | `HealthResponse { status: String }` |
| `environments_list()` | `GET /environments` | `Vec<Environment>` |
| `environment_get(id)` | `GET /environments/{id}` | `Environment` |
| `containers_list(env_id)` | `GET /environments/{id}/containers` | `Vec<Container>` |
| `container_get(env_id, container_id)` | `GET /environments/{id}/containers/{containerId}` | `Container` |
| `container_start(env_id, container_id)` | `POST /environments/{id}/containers/{containerId}/start` | `ContainerActionResult` |
| `container_stop(env_id, container_id)` | `POST /environments/{id}/containers/{containerId}/stop` | `ContainerActionResult` |
| `container_restart(env_id, container_id)` | `POST /environments/{id}/containers/{containerId}/restart` | `ContainerActionResult` |
| `container_redeploy(env_id, container_id)` | `POST /environments/{id}/containers/{containerId}/redeploy` | `ContainerActionResult` |
| `projects_list(env_id)` | `GET /environments/{envId}/projects` | `Vec<Project>` |
| `project_create(env_id, body)` | `POST /environments/{envId}/projects` | `Project` |
| `project_up(env_id, project_id)` | `POST /environments/{envId}/projects/{projId}/up` | `ProjectActionResult` |
| `project_down(env_id, project_id)` | `POST /environments/{envId}/projects/{projId}/down` | `ProjectActionResult` |
| `project_redeploy(env_id, project_id)` | `POST /environments/{envId}/projects/{projId}/redeploy` | `ProjectActionResult` |
| `volumes_list(env_id)` | `GET /environments/{envId}/volumes` | `Vec<Volume>` |
| `volume_delete(env_id, name)` | `DELETE /environments/{envId}/volumes/{name}` | `()` |
| `volumes_prune(env_id)` | `POST /environments/{envId}/volumes/prune` | `PruneResult` |
| `images_list(env_id)` | `GET /environments/{envId}/images` | `Vec<Image>` |
| `image_pull(env_id, image)` | `POST /environments/{envId}/images/pull` | `ImagePullResult` |
| `images_prune(env_id)` | `POST /environments/{envId}/images/prune` | `ImagePruneResult` |
| `image_update_summary(env_id)` | `GET /environments/{envId}/image-updates/summary` | `ImageUpdateSummary` |

## Action Catalog (dispatch layer)

The catalog is in `crates/lab/src/dispatch/arcane/catalog.rs`.

### Built-ins (every tool)

| Action | Params | Returns |
|--------|--------|---------|
| `help` | — | Catalog |
| `schema` | `action` (string, required) | Schema |

### Implemented Actions

| Action | SDK method | Destructive | MCP | CLI | API |
|--------|-----------|-------------|-----|-----|-----|
| `health` | `health()` | no | ✅ | ✅ | ✅ |
| `environment.list` | `environments_list()` | no | ✅ | ✅ | ✅ |
| `environment.get` | `environment_get(id)` | no | ✅ | ✅ | ✅ |
| `container.list` | `containers_list(env_id)` | no | ✅ | ✅ | ✅ |
| `container.get` | `container_get(env_id, container_id)` | no | ✅ | ✅ | ✅ |
| `container.start` | `container_start(env_id, container_id)` | no | ✅ | ✅ | ✅ |
| `container.stop` | `container_stop(env_id, container_id)` | no | ✅ | ✅ | ✅ |
| `container.restart` | `container_restart(env_id, container_id)` | no | ✅ | ✅ | ✅ |
| `container.redeploy` | `container_redeploy(env_id, container_id)` | **yes** | ✅ | ✅ | ✅ |
| `project.list` | `projects_list(env_id)` | no | ✅ | ✅ | ✅ |
| `project.create` | `project_create(env_id, body)` | no | ✅ | ✅ | ✅ |
| `project.up` | `project_up(env_id, project_id)` | no | ✅ | ✅ | ✅ |
| `project.down` | `project_down(env_id, project_id)` | **yes** | ✅ | ✅ | ✅ |
| `project.redeploy` | `project_redeploy(env_id, project_id)` | no | ✅ | ✅ | ✅ |
| `volume.list` | `volumes_list(env_id)` | no | ✅ | ✅ | ✅ |
| `volume.delete` | `volume_delete(env_id, volume_name)` | **yes** | ✅ | ✅ | ✅ |
| `volume.prune` | `volumes_prune(env_id)` | **yes** | ✅ | ✅ | ✅ |
| `image.list` | `images_list(env_id)` | no | ✅ | ✅ | ✅ |
| `image.pull` | `image_pull(env_id, image)` | no | ✅ | ✅ | ✅ |
| `image.prune` | `images_prune(env_id)` | **yes** | ✅ | ✅ | ✅ |
| `image.update-summary` | `image_update_summary(env_id)` | no | ✅ | ✅ | ✅ |

### Action Parameters

**`environment.get`**
| Param | Type | Required |
|-------|------|----------|
| `id` | string | yes |

**`container.list`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |

**`container.get`, `container.start`, `container.stop`, `container.restart`, `container.redeploy`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |
| `container_id` | string | yes |

**`project.list`, `volume.list`, `image.list`, `volume.prune`, `image.prune`, `image.update-summary`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |

**`project.create`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |
| `body` | object | yes |

**`project.up`, `project.down`, `project.redeploy`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |
| `project_id` | string | yes |

**`volume.delete`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |
| `volume_name` | string | yes |

**`image.pull`**
| Param | Type | Required |
|-------|------|----------|
| `env_id` | string | yes |
| `image` | string | yes |

Destructive actions: `container.redeploy`, `project.down`, `volume.delete`, `volume.prune`, `image.prune`.
MCP requires elicitation confirmation; CLI requires `-y` / `--yes`; API requires `"confirm": true` in params.

## Surface Details

### CLI (`lab arcane`)

Tier 2 (dispatch-backed thin shim). Accepts `action` as positional arg and `--params` as a
JSON string. Defaults to `help` when no action is given.

```
lab arcane health
lab arcane environment.list
lab arcane container.list --params '{"env_id":"abc123"}'
lab arcane container.redeploy --params '{"env_id":"abc123","container_id":"xyz"}'
```

No `-y` flag is wired in the current CLI shim (the CLI uses `run_action_command`, not
`run_confirmable_action_command`). Confirmation for `container.redeploy` is therefore only
enforced on the MCP and API surfaces.

### MCP

Single tool `arcane`. Thin adapter forwarding to `crate::dispatch::arcane::dispatch`.

### API (`POST /v1/arcane`)

Standard action+params envelope. Uses `AppState.clients.arcane` (pre-built at startup). The
`help` and `schema` built-ins are handled inline by `handle_action`; all other actions pass
through `dispatch_with_client`.

## Types

```rust
// HealthResponse
{ status: String }

// Environment
{ id, api_url, status, enabled, is_edge, name?, connected?, connected_at?, last_heartbeat? }

// Container
{ id, names: Vec<String>, image, image_id, command, created: i64, state, status, labels?, ports? }

// ContainerActionResult
{ success?: bool, message?: String }

// Project
{ id, name, status?, environment_id?, created_at?, updated_at? }

// ProjectActionResult
{ success?: bool, message?: String }

// Volume
{ name, driver?, mountpoint?, scope?, created_at?, labels?, options? }

// PruneResult
{ volumes_deleted?: Vec<String>, space_reclaimed?: i64 }

// Image
{ id, repo_tags?: Vec<String>, repo_digests?: Vec<String>, size?: i64, created?: i64, labels? }

// ImagePullResult
{ success?: bool, message?: String }

// ImagePruneResult
{ images_deleted?: Vec<Value>, space_reclaimed?: i64 }

// ImageUpdateSummary
{ updates_available?: i32, images?: Vec<Value> }
```

## Endpoint Inventory (upstream spec — 265 paths)

Rows marked ✅ are implemented. All others remain at ⬜ (spec inventory only).

| Method | Endpoint | Operation | Impl | MCP | CLI | API |
|--------|----------|-----------|------|-----|-----|-----|
| GET | /api-keys | list-api-keys | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /api-keys | create-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /api-keys/{id} | delete-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /api-keys/{id} | get-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /api-keys/{id} | update-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-images/favicon | get-favicon | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-images/logo | get-logo | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-images/logo-email | get-logo-email | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-images/profile | get-default-profile | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-images/pwa/{filename} | get-pwa-icon | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /app-version | getAppVersion | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/login | login | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/logout | logout | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /auth/me | get-current-user | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/password | change-password | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /auth/refresh | refresh-token | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /container-registries | listContainerRegistries | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /container-registries | createContainerRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /container-registries/sync | syncContainerRegistries | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /container-registries/{id} | deleteContainerRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /container-registries/{id} | getContainerRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /container-registries/{id} | updateContainerRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /container-registries/{id}/test | testContainerRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /customize/categories | get-customize-categories | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /customize/git-repositories | listGitRepositories | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /customize/git-repositories | createGitRepository | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /customize/git-repositories/{id} | deleteGitRepository | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /customize/git-repositories/{id} | getGitRepository | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /customize/git-repositories/{id} | updateGitRepository | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /customize/git-repositories/{id}/branches | listGitRepositoryBranches | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /customize/git-repositories/{id}/files | browseGitRepositoryFiles | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /customize/git-repositories/{id}/test | testGitRepository | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /customize/search | search-customize | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments | listEnvironments | ✅ | ✅ | ✅ | ✅ |
| POST | /environments | createEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/pair | pairEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id} | deleteEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id} | getEnvironment | ✅ | ✅ | ✅ | ✅ |
| PUT | /environments/{id} | updateEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/agent/pair | pairAgent | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/builds/browse | builds-browse-delete | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/builds/browse | builds-browse | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/builds/browse/content | builds-browse-content | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/builds/browse/download | builds-browse-download | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/builds/browse/mkdir | builds-browse-mkdir | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/builds/browse/upload | builds-browse-upload | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/containers | list-containers | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/containers | create-container | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/containers/counts | container-status-counts | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/containers/{containerId} | delete-container | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/containers/{containerId} | get-container | ✅ | ✅ | ✅ | ✅ |
| PUT | /environments/{id}/containers/{containerId}/auto-update | set-container-auto-update | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/containers/{containerId}/redeploy | redeploy-container | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/containers/{containerId}/restart | restart-container | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/containers/{containerId}/start | start-container | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/containers/{containerId}/stop | stop-container | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/containers/{containerId}/update | update-container | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/dashboard | get-dashboard | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/dashboard/action-items | get-dashboard-action-items | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/deployment | getDeploymentSnippets | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/gitops-syncs | listGitOpsSyncs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/gitops-syncs | createGitOpsSync | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/gitops-syncs/import | importGitOpsSyncs | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/gitops-syncs/{syncId} | deleteGitOpsSync | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/gitops-syncs/{syncId} | getGitOpsSync | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/gitops-syncs/{syncId} | updateGitOpsSync | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/gitops-syncs/{syncId}/files | browseGitOpsSyncFiles | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/gitops-syncs/{syncId}/status | getGitOpsSyncStatus | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/gitops-syncs/{syncId}/sync | performGitOpsSync | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/heartbeat | updateHeartbeat | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/image-updates/check | check-image-update | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/image-updates/check-all | check-all-images | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/image-updates/check-batch | check-multiple-images | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/image-updates/check/{imageId} | check-image-update-by-id | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/image-updates/check/{imageId} | check-image-update-by-id-post | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/image-updates/summary | get-update-summary | ✅ | ✅ | ✅ | ✅ |
| GET | /environments/{id}/images | list-images | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/images/build | build-image | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/builds | list-image-builds | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/builds/{buildId} | get-image-build | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/counts | get-image-usage-counts | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/images/prune | prune-images | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/images/pull | pull-image | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/images/upload | upload-image | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/images/vulnerabilities/summaries | get-image-vulnerability-summaries | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/images/{imageId} | remove-image | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/{imageId} | get-image | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/{imageId}/vulnerabilities | get-image-vulnerabilities | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/{imageId}/vulnerabilities/list | list-image-vulnerabilities | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/images/{imageId}/vulnerabilities/scan | scan-image-vulnerabilities | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/images/{imageId}/vulnerabilities/summary | get-image-vulnerability-summary | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/job-schedules | get-job-schedules | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/job-schedules | update-job-schedules | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/jobs | list-jobs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/jobs/{jobId}/run | run-job | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/networks | list-networks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/networks | create-network | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/networks/counts | network-counts | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/networks/prune | prune-networks | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/networks/topology | get-network-topology | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/networks/{networkId} | delete-network | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/networks/{networkId} | get-network | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/notifications/apprise | get-apprise-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/notifications/apprise | create-or-update-apprise-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/notifications/apprise/test | test-apprise-notification | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/notifications/settings | get-all-notification-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/notifications/settings | create-or-update-notification-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/notifications/settings/{provider} | delete-notification-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/notifications/settings/{provider} | get-notification-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/notifications/test/{provider} | test-notification | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/ports | list-ports | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/projects | list-projects | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/projects | create-project | ✅ | ✅ | ✅ | ✅ |
| GET | /environments/{id}/projects/counts | get-project-status-counts | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/projects/{projectId} | get-project | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/projects/{projectId} | update-project | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/projects/{projectId}/build | build-project-images | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/projects/{projectId}/destroy | destroy-project | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/projects/{projectId}/down | down-project | ✅ | ✅ | ✅ | ✅ |
| PUT | /environments/{id}/projects/{projectId}/includes | update-project-include | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/projects/{projectId}/pull | pull-project-images | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/projects/{projectId}/redeploy | redeploy-project | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/projects/{projectId}/restart | restart-project | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/projects/{projectId}/up | deploy-project | ✅ | ✅ | ✅ | ✅ |
| GET | /environments/{id}/settings | get-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/settings | update-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/settings/public | get-public-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/configs | list-swarm-configs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/configs | create-swarm-config | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/swarm/configs/{configId} | delete-swarm-config | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/configs/{configId} | get-swarm-config | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/swarm/configs/{configId} | update-swarm-config | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/info | get-swarm-info | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/init | init-swarm | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/join | join-swarm | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/join-tokens | get-swarm-join-tokens | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/join-tokens/rotate | rotate-swarm-join-tokens | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/leave | leave-swarm | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/nodes | list-swarm-nodes | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/swarm/nodes/{nodeId} | delete-swarm-node | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/nodes/{nodeId} | get-swarm-node | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /environments/{id}/swarm/nodes/{nodeId} | update-swarm-node | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/nodes/{nodeId}/agent/deployment | get-swarm-node-agent-deployment | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/nodes/{nodeId}/demote | demote-swarm-node | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/nodes/{nodeId}/promote | promote-swarm-node | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/nodes/{nodeId}/tasks | list-swarm-node-tasks | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/secrets | list-swarm-secrets | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/secrets | create-swarm-secret | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/swarm/secrets/{secretId} | delete-swarm-secret | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/secrets/{secretId} | get-swarm-secret | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/swarm/secrets/{secretId} | update-swarm-secret | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/services | list-swarm-services | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/services | create-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/swarm/services/{serviceId} | delete-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/services/{serviceId} | get-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/swarm/services/{serviceId} | update-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/services/{serviceId}/rollback | rollback-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/services/{serviceId}/scale | scale-swarm-service | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/services/{serviceId}/tasks | list-swarm-service-tasks | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/swarm/spec | update-swarm-spec | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/stacks | list-swarm-stacks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/stacks | deploy-swarm-stack | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/stacks/config/render | render-swarm-stack-config | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/swarm/stacks/{name} | delete-swarm-stack | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/stacks/{name} | get-swarm-stack | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/stacks/{name}/services | list-swarm-stack-services | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/stacks/{name}/source | get-swarm-stack-source | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/swarm/stacks/{name}/source | update-swarm-stack-source | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/stacks/{name}/tasks | list-swarm-stack-tasks | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/status | get-swarm-status | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/tasks | list-swarm-tasks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/swarm/unlock | unlock-swarm | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/swarm/unlock-key | get-swarm-unlock-key | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/sync | syncEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/containers/start-all | start-all-containers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/containers/start-stopped | start-all-stopped-containers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/containers/stop-all | stop-all-containers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/convert | convert-docker-run | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/system/docker/info | get-docker-info | ⬜ | ⬜ | ⬜ | ⬜ |
| HEAD | /environments/{id}/system/health | system-health | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/prune | prune-all | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/system/upgrade | trigger-upgrade | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/system/upgrade/check | check-upgrade | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/templates/variables | getGlobalVariables | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /environments/{id}/templates/variables | updateGlobalVariables | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/test | testConnection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/updater/history | get-updater-history | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/updater/run | run-updater | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/updater/status | get-updater-status | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/version | getEnvironmentVersion | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes | list-volumes | ✅ | ✅ | ✅ | ✅ |
| POST | /environments/{id}/volumes | create-volume | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/volumes/backups/{backupId} | delete-volume-backup | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/backups/{backupId}/download | download-volume-backup | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/backups/{backupId}/files | list-backup-files | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/backups/{backupId}/has-path | backup-has-path | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/counts | get-volume-usage-counts | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/prune | prune-volumes | ✅ | ✅ | ✅ | ✅ |
| GET | /environments/{id}/volumes/sizes | get-volume-sizes | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/volumes/{volumeName} | remove-volume | ✅ | ✅ | ✅ | ✅ |
| GET | /environments/{id}/volumes/{volumeName} | get-volume | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/{volumeName}/backups | list-volume-backups | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/backups | create-volume-backup | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/backups/upload | upload-volume-backup | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/backups/{backupId}/restore | restore-volume-backup | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/backups/{backupId}/restore-files | restore-volume-backup-files | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/volumes/{volumeName}/browse | delete-volume-file | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/{volumeName}/browse | browse-volume-directory | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/{volumeName}/browse/content | get-volume-file-content | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/{volumeName}/browse/download | download-volume-file | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/browse/mkdir | create-volume-directory | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/volumes/{volumeName}/browse/upload | upload-volume-file | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/volumes/{volumeName}/usage | get-volume-usage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/vulnerabilities/all | list-environment-vulnerabilities | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/vulnerabilities/ignore | ignore-vulnerability | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/vulnerabilities/ignore/{ignoreId} | unignore-vulnerability | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/vulnerabilities/ignored | list-ignored-vulnerabilities | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/vulnerabilities/image-options | list-environment-vulnerability-image-options | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/vulnerabilities/scanner-status | get-vulnerability-scanner-status | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/vulnerabilities/summary | get-environment-vulnerability-summary | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /environments/{id}/webhooks | list-webhooks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /environments/{id}/webhooks | create-webhook | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /environments/{id}/webhooks/{webhookId} | delete-webhook | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /environments/{id}/webhooks/{webhookId} | update-webhook | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /events | listEvents | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /events | createEvent | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /events/environment/{environmentId} | getEventsByEnvironment | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /events/{eventId} | deleteEvent | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fonts/mono | get-mono-font | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fonts/sans | get-sans-font | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fonts/serif | get-serif-font | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /git-repositories/sync | syncGitRepositories | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /health | health-check | ✅ | ✅ | ✅ | ✅ |
| HEAD | /health | health-check-head | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /notifications/dispatch | dispatch-notification | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /oidc/callback | handle-oidc-callback | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /oidc/config | get-oidc-config | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /oidc/device/code | initiate-oidc-device-auth | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /oidc/device/token | exchange-oidc-device-token | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /oidc/status | get-oidc-status | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /oidc/url | get-oidc-auth-url | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /settings/categories | get-settings-categories | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /settings/search | search-settings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /swarm/node-identity | get-swarm-node-identity | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates | listTemplatesPaginated | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /templates | createTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/all | getAllTemplates | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/default | getDefaultTemplates | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /templates/default | saveDefaultTemplates | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/fetch | fetchTemplateRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/registries | getTemplateRegistries | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /templates/registries | createTemplateRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /templates/registries/{id} | deleteTemplateRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /templates/registries/{id} | updateTemplateRegistry | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /templates/{id} | deleteTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/{id} | getTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /templates/{id} | updateTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /templates/{id}/content | getTemplateContent | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /templates/{id}/download | downloadTemplate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /users | listUsers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users | createUser | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /users/{userId} | deleteUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /users/{userId} | getUser | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /users/{userId} | updateUser | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /version | getVersion | ⬜ | ⬜ | ⬜ | ⬜ |
