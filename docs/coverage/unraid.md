# Unraid Coverage

**Last updated:** 2026-04-14
**Source spec:** GraphQL introspection against tootie (10-1-0-2:31337/graphql)
**Auth:** `X-API-Key` header

## Implementation Status

| Surface | Status |
|---------|--------|
| SDK (lab-apis) | Complete |
| Dispatch layer | Complete |
| CLI shim | Complete (Tier-2) |
| MCP dispatcher | Complete |
| API handler | Complete |
| Health check | Complete |
| TUI metadata | Complete |

## Action Coverage

30 total actions: 1 built-in (`help`) + 29 service actions. The built-in `help` and `schema` actions are handled by the dispatcher before reaching service-specific logic. They are available on all surfaces (MCP, CLI, API).

| Action | Client method | Destructive | MCP | CLI | API | Returns |
|--------|---------------|-------------|-----|-----|-----|---------|
| `help` | (built-in) | No | Yes | Yes | Yes | Action catalog |
| `system.info` | `system_info()` | No | Yes | Yes | Yes | `SystemInfo` (OS, CPU, system, versions) |
| `system.metrics` | `system_metrics()` | No | Yes | Yes | Yes | `SystemMetrics` (CPU %, memory total/used/free/%) |
| `system.array` | `system_array()` | No | Yes | Yes | Yes | `ArrayStatus` (state, disks, parities, caches) |
| `system.online` | `system_online()` | No | Yes | Yes | Yes | `{ online: bool }` |
| `docker.list` | `docker_list()` | No | Yes | Yes | Yes | `Vec<DockerContainer>` |
| `docker.start` | `docker_start(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `docker.stop` | `docker_stop(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `docker.restart` | `docker_restart(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `disk.list` | `disk_list()` | No | Yes | Yes | Yes | `Vec<DiskInfo>` |
| `vm.list` | `vm_list()` | No | Yes | Yes | Yes | `Vec<VmInfo>` (id, name, status, uuid, cores, memory) |
| `vm.start` | `vm_start(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `vm.stop` | `vm_stop(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` — may corrupt unsaved VM work |
| `vm.pause` | `vm_pause(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `vm.resume` | `vm_resume(id)` | No | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `notification.list` | `notification_list()` | No | Yes | Yes | Yes | `Vec<NotificationInfo>` |
| `notification.create` | `notification_create(title, description?, importance?, type?)` | No | Yes | Yes | Yes | `{ ok: true }` |
| `notification.archive` | `notification_archive(id)` | Yes | Yes | Yes | Yes | `{ ok: true, id: string }` |
| `parity.history` | `parity_history()` | No | Yes | Yes | Yes | `Vec<ParityHistoryEntry>` |
| `parity.check-start` | `parity_check_start(correcting?)` | No | Yes | Yes | Yes | `{ ok: true }` |
| `parity.check-pause` | `parity_check_pause()` | No | Yes | Yes | Yes | `{ ok: true }` |
| `parity.check-cancel` | `parity_check_cancel()` | Yes | Yes | Yes | Yes | `{ ok: true }` — invalidates partial parity data |
| `share.list` | `share_list()` | No | Yes | Yes | Yes | `Vec<Share>` |
| `plugin.list` | `plugin_list()` | No | Yes | Yes | Yes | `Vec<Plugin>` |
| `network.list` | `network_list()` | No | Yes | Yes | Yes | `NetworkInfo` (interfaces, IPs, MAC, MTU, speed) |
| `ups.devices` | `ups_devices()` | No | Yes | Yes | Yes | `Vec<UpsDevice>` |
| `ups.config` | `ups_config()` | No | Yes | Yes | Yes | `UpsConfig` |
| `log.read` | `log_read(path, lines?)` | No | Yes | Yes | Yes | `{ content: string }` |
| `flash.status` | `flash_status()` | No | Yes | Yes | Yes | `FlashStatus` (guid, product, vendor, size, state) |
| `flash.backup` | `flash_backup()` | Yes | Yes | Yes | Yes | `{ ok: true }` — overwrites existing backup |

Docker mutations (`docker.start`, `docker.stop`, `docker.restart`) require `id: string`
(the container's prefixed ID, e.g. `docker_container:abc123`).

VM mutations (`vm.start`, `vm.stop`, `vm.pause`, `vm.resume`) require `id: string`.
`notification.archive` requires `id: string`.
`log.read` requires `path: string`; `lines: integer` is optional (default 50).

## GraphQL Queries

| Action | Query / Mutation |
|--------|-----------------|
| `system.online` | `query { online }` |
| `system.info` | `query { info { id os cpu system versions } }` |
| `system.metrics` | `query { metrics { id cpu { id percentTotal } memory { id total used free percentTotal } } }` |
| `system.array` | `query { array { id state disks parities caches } }` |
| `docker.list` | `query { docker { id containers { id names image created state status autoStart ports lanIpPorts } } }` |
| `docker.start` | `mutation DockerStart($id: PrefixedID!) { docker { start(id: $id) { id names image state status autoStart } } }` |
| `docker.stop` | `mutation DockerStop($id: PrefixedID!) { docker { stop(id: $id) { id names image state status autoStart } } }` |
| `docker.restart` | sequential `docker_stop` + `docker_start` (no native mutation) |
| `disk.list` | `query { disks { id name device vendor size type smartStatus temperature serialNum } }` |
| `vm.list` | `query { vms { id name status uuid cores memory } }` |
| `vm.start` | `mutation VmAction($id: String!, $action: VmAction!) { vmAction(id: $id, action: START) }` |
| `vm.stop` | `mutation VmAction($id: String!, $action: VmAction!) { vmAction(id: $id, action: STOP) }` |
| `vm.pause` | `mutation VmAction($id: String!, $action: VmAction!) { vmAction(id: $id, action: PAUSE) }` |
| `vm.resume` | `mutation VmAction($id: String!, $action: VmAction!) { vmAction(id: $id, action: RESUME) }` |
| `notification.list` | `query { notifications { id title description importance type timestamp } }` |
| `notification.create` | `mutation CreateNotification($title, $description, $importance, $type) { createNotification(...) }` |
| `notification.archive` | `mutation ArchiveNotification($id: String!) { archiveNotification(id: $id) }` |
| `parity.history` | `query { parityHistory { date duration speed status errors } }` |
| `parity.check-start` | `mutation ParityCheck($action: ParityCheckAction!, $correcting: Boolean) { parityCheck(action: START, ...) }` |
| `parity.check-pause` | `mutation ParityCheck($action: ParityCheckAction!) { parityCheck(action: PAUSE) }` |
| `parity.check-cancel` | `mutation ParityCheck($action: ParityCheckAction!) { parityCheck(action: CANCEL) }` |
| `share.list` | `query { shares { id name comment allocator splitLevel useCache exportEnabled security } }` |
| `plugin.list` | `query { installedUnraidPlugins { id name version author description updateAvailable } }` |
| `network.list` | `query { network { id interfaces { id name ipv4 ipv6 macAddress connected mtu speed } } }` |
| `ups.devices` | `query { upsDevices { id name status batteryCharge batteryRuntime model driver } }` |
| `ups.config` | `query { upsConfiguration { id upsName driver port pollInterval } }` |
| `log.read` | `query LogFile($path: String!, $lines: Int) { logFile(path: $path, lines: $lines) }` |
| `flash.status` | `query { flash { id guid product vendor size state } }` |
| `flash.backup` | `mutation { initiateFlashBackup }` |

## Files

| File | Purpose |
|------|---------|
| `crates/lab-apis/src/unraid.rs` | Module declaration, `META`, `ServiceClient` impl |
| `crates/lab-apis/src/unraid/client.rs` | `UnraidClient` — 29 async methods (system, docker, disk, vm, notification, parity, share, plugin, network, ups, log, flash) |
| `crates/lab-apis/src/unraid/types.rs` | Request/response types (serde) |
| `crates/lab-apis/src/unraid/error.rs` | `UnraidError` (wraps `ApiError`) |
| `crates/lab/src/dispatch/unraid.rs` | Module declaration (re-exports `ACTIONS`, `dispatch`, `dispatch_with_client`, `client_from_env`, `client_from_instance`, `not_configured_error`) |
| `crates/lab/src/dispatch/unraid/catalog.rs` | `ACTIONS` — 30 total (1 built-in `help` + 29 service actions); 9 destructive (docker.start/stop/restart, vm.start/stop/pause, notification.archive, parity.check-cancel, flash.backup) |
| `crates/lab/src/dispatch/unraid/client.rs` | `client_from_env()`, `client_from_instance()`, `require_client()`, `not_configured_error()`, URL normaliser, module-level `InstancePool` |
| `crates/lab/src/dispatch/unraid/dispatch.rs` | `dispatch()` + `dispatch_with_client()` — routes all 29 service actions + built-in help/schema |
| `crates/lab/src/dispatch/unraid/params.rs` | `require_id()`, `require_path()`, `require_title()`, `optional_lines()`, `optional_correcting()`, `optional_description()`, `optional_importance()`, `optional_type()` param extractors |
| `crates/lab-apis/tests/unraid_client.rs` | wiremock-based unit tests (coverage for SDK methods) |
| `crates/lab/src/cli/unraid.rs` | CLI shim (Tier-2: `--params` JSON object, `--instance`, `-y`/`--yes`) |
| `crates/lab/src/mcp/registry.rs` | MCP service registration entry for unraid (wires directly to `dispatch::unraid::dispatch`) |
| `crates/lab/src/api/services/unraid.rs` | Axum route group → `dispatch_with_client` (uses `AppState.clients.unraid`; falls back to `client_from_instance` for labelled requests) |
| `crates/lab/src/cli/health.rs` | `unraid_row()` health probe |
| `crates/lab/src/tui/metadata.rs` | TUI plugin metadata block |

## Multi-Instance Support

The dispatch client layer uses `InstancePool<UnraidClient>` backed by a `OnceLock`. Env
vars follow the pattern `UNRAID_URL` / `UNRAID_API_KEY` (default instance) and
`UNRAID_<LABEL>_URL` / `UNRAID_<LABEL>_API_KEY` (named instances). Select via
`params.instance` (MCP/API) or `--instance` (CLI).

`client_from_instance` returns `Arc<UnraidClient>`; the API handler uses the pre-built
`AppState.clients.unraid` for the default instance and calls `client_from_instance` only
for labelled requests.

## URL Normalisation

The dispatch client normalises the stored URL by stripping any trailing `/graphql` path
segment and surrounding slashes. Both `https://host:31337` and `https://host:31337/graphql`
are accepted in `UNRAID_URL`; the client always appends `/graphql` itself.

## Config

| Env var | Required | Purpose |
|---------|----------|---------|
| `UNRAID_URL` | Yes | Base URL of the Unraid server (with or without `/graphql` suffix) |
| `UNRAID_API_KEY` | Yes | API key sent as `X-API-Key` header |

The header name is `X-API-Key` (mixed case). Unraid's server validates the exact casing —
this is an intentional deviation from the dispatch template's `X-Api-Key` default.

## Known Limitations

- **No in-process rate limiting** — Unraid enforces approximately 100 requests per 10 seconds; callers must stay within this bound.
- **`docker.restart` implementation** — implemented as a sequential `docker_stop` + `docker_start` because no native restart mutation exists in the schema. This operation is **not cancellation-safe**: if the future is dropped after `stop` completes but before `start` is issued, the container remains permanently stopped.
- **CLI Tier-2 only** — raw `--params` JSON object; typed Tier-1 subcommands are deferred.
- **GraphQL subscriptions deferred** — 17 subscriptions available via `graphql-transport-ws` are not yet implemented; reserved for a separate epic.
- **MCP registry wiring** — `crates/lab/src/registry.rs` does not exist as a separate file; the MCP surface is wired directly in `crates/lab/src/mcp/registry.rs` using the shared dispatch layer.
