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

The built-in `help` and `schema` actions are handled by the dispatcher before reaching
service-specific logic. They are not listed as service actions in the catalog but are
available on all surfaces.

| Action | Client method | Destructive | Returns |
|--------|---------------|-------------|---------|
| `system.info` | `system_info()` | No | `SystemInfo` (OS, CPU, system, versions) |
| `system.metrics` | `system_metrics()` | No | `SystemMetrics` (CPU %, memory total/used/free/%) |
| `system.array` | `system_array()` | No | `ArrayStatus` (state, disks, parities, caches) |
| `system.online` | `system_online()` | No | `{ online: bool }` |
| `docker.list` | `docker_list()` | No | `Vec<DockerContainer>` |
| `docker.start` | `docker_start(id)` | Yes | `{ ok: true, id: string }` |
| `docker.stop` | `docker_stop(id)` | Yes | `{ ok: true, id: string }` |
| `docker.restart` | `docker_restart(id)` | Yes | `{ ok: true, id: string }` |
| `disk.list` | `disk_list()` | No | `Vec<DiskInfo>` |
| `vm.list` | `vm_list()` | No | `Vec<VmInfo>` (id, name, status, uuid, cores, memory) |
| `vm.start` | `vm_start(id)` | Yes | `{ ok: true, id: string }` |
| `vm.stop` | `vm_stop(id)` | **Yes** | `{ ok: true, id: string }` — may corrupt unsaved VM work |
| `vm.pause` | `vm_pause(id)` | Yes | `{ ok: true, id: string }` |
| `vm.resume` | `vm_resume(id)` | No | `{ ok: true, id: string }` |
| `notification.list` | `notification_list()` | No | `Vec<NotificationInfo>` |
| `notification.create` | `notification_create(title, description?, importance?, type?)` | No | `{ ok: true }` |
| `notification.archive` | `notification_archive(id)` | Yes | `{ ok: true, id: string }` |
| `parity.history` | `parity_history()` | No | `Vec<ParityHistoryEntry>` |
| `parity.check-start` | `parity_check_start(correcting?)` | No | `{ ok: true }` |
| `parity.check-pause` | `parity_check_pause()` | No | `{ ok: true }` |
| `parity.check-cancel` | `parity_check_cancel()` | **Yes** | `{ ok: true }` — invalidates partial parity data |
| `share.list` | `share_list()` | No | `Vec<Share>` |
| `plugin.list` | `plugin_list()` | No | `Vec<Plugin>` |
| `network.list` | `network_list()` | No | `NetworkInfo` (interfaces, IPs, MAC, MTU, speed) |
| `ups.devices` | `ups_devices()` | No | `Vec<UpsDevice>` |
| `ups.config` | `ups_config()` | No | `UpsConfig` |
| `log.read` | `log_read(path, lines?)` | No | `{ content: string }` |
| `flash.status` | `flash_status()` | No | `FlashStatus` (guid, product, vendor, size, state) |
| `flash.backup` | `flash_backup()` | **Yes** | `{ ok: true }` — overwrites existing backup |

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
| `crates/lab-apis/src/unraid/client.rs` | `UnraidClient` — 29 async methods |
| `crates/lab-apis/src/unraid/types.rs` | Request/response types (serde) |
| `crates/lab-apis/src/unraid/error.rs` | `UnraidError` (wraps `ApiError`) |
| `crates/lab/src/dispatch/unraid.rs` | Module declaration (re-exports `ACTIONS`, `dispatch`, `dispatch_with_client`, `client_from_env`, `client_from_instance`, `not_configured_error`) |
| `crates/lab/src/dispatch/unraid/catalog.rs` | `ACTIONS` — 29 service actions + `help` built-in (30 total); 8 destructive |
| `crates/lab/src/dispatch/unraid/client.rs` | `client_from_env()`, `client_from_instance()`, `require_client()`, `not_configured_error()`, URL normaliser, module-level `InstancePool` |
| `crates/lab/src/dispatch/unraid/dispatch.rs` | `dispatch()` + `dispatch_with_client()` |
| `crates/lab/src/dispatch/unraid/params.rs` | `require_id()`, `require_path()`, `require_title()`, `optional_lines()`, `optional_correcting()`, `optional_description()`, `optional_importance()`, `optional_type()` param extractors |
| `crates/lab-apis/tests/unraid_client.rs` | wiremock-based unit tests for all 20 new SDK methods (21 tests total) |
| `crates/lab/src/cli/unraid.rs` | CLI shim (Tier-2: `--params` JSON object, `--instance`, `-y`/`--yes`) |
| `crates/lab/src/mcp/services/unraid.rs` | (file absent — MCP registry wires directly to `dispatch::unraid::dispatch`) |
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

- No in-process rate limiting — Unraid enforces approximately 100 req/10 s; callers must
  stay within this bound.
- `docker.restart` is implemented as a sequential `docker_stop` + `docker_start` because no
  native restart mutation exists in the schema. This operation is **not cancellation-safe**:
  if the future is dropped after `stop` completes but before `start` is issued, the container
  remains permanently stopped.
- Tier-2 CLI only (raw `--params` JSON object); typed Tier-1 subcommands are deferred.
- GraphQL subscriptions (17 available via `graphql-transport-ws`) are deferred to a separate
  epic.
- `mcp/services/unraid.rs` does not exist; the MCP surface is wired directly in the registry.
