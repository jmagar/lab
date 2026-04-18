# Deploy Service Design

## Decision

Complete and fix the partially-implemented synthetic non-HTTP service named
`deploy` that rolls out the local `lab` binary to one, many, or all
SSH-discovered devices. The service skeleton (dispatch layer, types, error
taxonomy, CLI shim, and MCP adapter) exists in the codebase; this spec
describes the remaining work needed to make the execution pipeline functional.

`deploy` follows the same architectural pattern as `extract`:

- core capability logic lives in `lab-apis`
- CLI, MCP, and API stay thin
- the shared `dispatch/deploy/` layer owns action metadata, param coercion,
  config reads, and surface-neutral execution

Unlike `extract`, `deploy` is not part of the bootstrap flow and should not
reuse `Category::Bootstrap`. It should use a new operator-oriented category or
the nearest existing non-bootstrap category if the metadata enum is not yet
being extended in the same change.

## Why

Operators already have:

- a local build environment
- a fleet of hosts in `~/.ssh/config`
- a device-runtime model documented in `docs/DEPLOY.md`

What is missing is a first-class rollout command that:

- builds the current repo artifact
- resolves SSH aliases and lab-defined groups
- copies the artifact safely
- swaps it atomically
- optionally restarts a service
- reports structured per-host outcomes

This is not an upstream API integration. It is a product capability with
real business logic, so it belongs in the synthetic-service class rather than a
thin CLI wrapper around ad hoc shell commands.

## Scope

### In scope for v1

- build the local `lab` release binary once with
  `cargo build --release --all-features -p lab`
- resolve one or more rollout targets from SSH aliases plus lab-defined groups
- support `all` targets
- run per-host standard preflight checks
- transfer with `rsync` preferred and SSH streaming fallback
- stage and install atomically with a timestamped backup
- optionally restart a configured `systemd` service
- verify binary presence and optional service status after rollout
- expose the same semantics over CLI, MCP, and HTTP API
- emit rollout-scoped observability at the real execution boundaries

### Out of scope for v1

- cross-compilation
- multi-architecture build matrices
- automatic rollback
- generic arbitrary build commands
- arbitrary artifact deployment from day one
- continuous device online or offline tracking
- post-restart application-level health probing beyond service status and basic
  binary verification

The ongoing online or offline tracking concern is intentionally deferred to a
separate `devices` capability.

## User Surface

The initial action catalog is:

- `help`
- `schema`
- `targets.list`
- `groups.list`
- `plan`
- `run`
- `verify`

### Action semantics

`targets.list`

- enumerates rollout-eligible SSH aliases discovered from `~/.ssh/config`
- returns alias, resolved hostname when known, and any deploy-specific host
  overrides from config

`groups.list`

- returns configured deploy groups from lab config
- expands each group to concrete SSH aliases

`plan`

- resolves targets from `targets`, `group`, or `all`
- merges deploy defaults with per-host overrides
- returns the intended rollout plan without building or mutating anything

`run`

- performs the full rollout path
- is `destructive: true`
- supports `dry_run` as a no-mutate path that still resolves targets and runs
  as much preflight as possible

`verify`

- re-checks target state after a rollout
- does not mutate remote state

### CLI surface

The human-facing CLI should be typed and thin. The exact syntax can be refined
in the planning phase, but the intended shape is:

```bash
lab deploy targets list
lab deploy groups list
lab deploy plan --group servers
lab deploy run --targets mini1,mini2 -y
lab deploy run --all -y
lab deploy verify --group servers
```

### MCP and API surface

MCP and API expose the service as one tool and one endpoint with action-based
dispatch:

```json
{"action":"plan","params":{"group":"servers"}}
{"action":"run","params":{"targets":["mini1","mini2"]}}
{"action":"verify","params":{"all":true}}
```

`run` must honor the existing destructive-operation contract:

- CLI requires confirmation bypass via `-y`
- API requires `params.confirm = true`
- MCP uses the shared `ActionSpec.destructive` metadata and existing dispatcher
  behavior

## Config Model

SSH inventory remains the discovery source. Deploy intent belongs in lab config.

### Discovery source

`~/.ssh/config` is the source for:

- valid target aliases
- SSH hostname overrides when present
- SSH user or port if the transport layer needs them

`deploy` must not depend on comments or naming conventions embedded in SSH
 config to infer groups or behavior.

### Lab-owned deploy config

Rollout preferences live in `config.toml`.

Proposed shape:

```toml
[deploy.defaults]
remote_path = "/usr/local/bin/lab"
service = "lab"
service_scope = "system"
restart = true
backup = true
verify_service = true

[deploy.groups]
servers = ["mini1", "mini2", "nas"]
edge = ["rpi-a", "rpi-b"]

[deploy.hosts.nas]
remote_path = "/opt/lab/bin/lab"
restart = false

[deploy.hosts.mini2]
service = "lab-worker"
service_scope = "user"
```

Rules:

- `deploy.defaults` is optional; built-in defaults still exist
- host overrides are keyed by SSH alias
- groups only expand to valid SSH aliases
- unknown aliases in a group are validation errors
- secret material does not belong in deploy config

This keeps `deploy` aligned with `docs/CONFIG.md`:

- URLs and secrets stay in env
- behavior and preferences live in TOML

## Module Layout

### `lab-apis`

`deploy` is a synthetic capability module with the standard service shape plus
supporting helpers:

- `crates/lab-apis/src/deploy.rs`
- `crates/lab-apis/src/deploy/client.rs`
- `crates/lab-apis/src/deploy/types.rs`
- `crates/lab-apis/src/deploy/error.rs`
- `crates/lab-apis/src/deploy/build.rs`
- `crates/lab-apis/src/deploy/ssh_config.rs`
- `crates/lab-apis/src/deploy/transport.rs`
- `crates/lab-apis/src/deploy/remote.rs`

Responsibilities:

- `client.rs`: top-level orchestration of plan, run, and verify workflows
- `types.rs`: rollout requests, target resolution, per-host outcomes, summaries
- `error.rs`: typed deployment failures
- `build.rs`: local build invocation and artifact discovery
- `ssh_config.rs`: SSH alias inventory loading
- `transport.rs`: transfer strategy abstraction with rsync-first fallback
- `remote.rs`: preflight, install, restart, and verification primitives

### `dispatch`

`deploy` follows the required directory-first dispatch shape:

- `crates/lab/src/dispatch/deploy.rs`
- `crates/lab/src/dispatch/deploy/catalog.rs`
- `crates/lab/src/dispatch/deploy/client.rs`
- `crates/lab/src/dispatch/deploy/params.rs`
- `crates/lab/src/dispatch/deploy/dispatch.rs`

Responsibilities:

- `catalog.rs`: single source of truth for `ACTIONS`
- `client.rs`: config loading, group expansion helpers, and any structured
  `not_configured_error()` equivalent if needed
- `params.rs`: all coercion from `serde_json::Value` to typed deploy requests
- `dispatch.rs`: `dispatch()` and `dispatch_with_client()`

Unlike an HTTP-backed service, `dispatch/deploy/client.rs` is not primarily an
env-to-client constructor. It is the product-layer resolution point for deploy
configuration and target policy.

### Product surfaces

Thin adapters:

- `crates/lab/src/cli/deploy.rs`
- `crates/lab/src/mcp/services/deploy.rs`
- `crates/lab/src/api/services/deploy.rs`

Required registration and metadata updates:

- `crates/lab-apis/src/lib.rs`
- `crates/lab/Cargo.toml`
- `crates/lab/src/cli.rs`
- `crates/lab/src/registry.rs`
- `crates/lab/src/mcp/services.rs`
- `crates/lab/src/api/services.rs`
- `crates/lab/src/api/router.rs`
- `crates/lab/src/api/state.rs`
- `crates/lab/src/tui/metadata.rs`

## Execution Flow

`run` executes through one shared path regardless of surface:

1. resolve targets from `targets`, `group`, or `all`
2. merge defaults with per-host overrides
3. build the local artifact once
4. run per-host preflight
5. transfer the artifact
6. install atomically
7. optionally restart the configured service
8. verify basic post-install state
9. return per-host and aggregate results

### Build contract

V1 builds exactly one local artifact:

```bash
cargo build --release --all-features -p lab
```

The output artifact is the local host build product. Hosts that do not match
the artifact's expected runtime environment fail preflight.

### Preflight contract

Each target should validate:

- remote host is reachable over SSH
- remote OS and architecture are compatible enough for this v1 model
- destination parent exists or can be created
- destination path is writable
- sufficient free space exists
- the configured service exists if restart is enabled
- at least one transfer mode is available

Preflight is standard, not strict:

- it catches obvious operator errors
- it does not attempt deep health modeling or rollback planning

### Transfer contract

Transfer strategy is hybrid:

- prefer `rsync`
- fall back to SSH streaming when `rsync` is unavailable or unsupported

This choice should be represented in the per-host result as `transport_used`.

### Install contract

Remote install is atomic:

1. upload to a temporary path
2. verify size or hash
3. move the current binary to a timestamped backup path if present
4. rename the staged binary into place
5. set executable mode

This is the destructive boundary of the service.

### Restart contract

If restart is enabled for a target:

- restart a configured `systemd` service
- support user or system scope through explicit config

The config model should not assume every rollout target restarts the same
service name or service scope.

### Verify contract

V1 verification is intentionally shallow and deterministic:

- binary exists at the target path
- optional service status command succeeds when restart was requested

V1 does not perform application-specific health checks or automatic rollback.

## Result Model

Every rollout returns:

- the resolved target set
- the artifact identity
- per-host results
- an overall summary

Per-host fields should include:

- `target`
- `ssh_host`
- `status`
- `stage`
- `warnings`
- `transport_used`
- `backup_path`
- `service_restarted`
- `message`

Suggested `stage` values:

- `resolve`
- `build`
- `preflight`
- `transfer`
- `install`
- `restart`
- `verify`

Suggested `status` values:

- `planned`
- `skipped`
- `ok`
- `failed`

## Error Model

Stable user-facing kinds should include:

- `unknown_target`
- `unknown_group`
- `no_targets_resolved`
- `build_failed`
- `preflight_failed`
- `transfer_failed`
- `install_failed`
- `restart_failed`
- `verification_failed`

These kinds should preserve the shared `ToolError` transport contract. The
service-specific SDK error can remain more detailed internally, but dispatch
must map outcomes into the existing stable envelope model.

Failure handling is per-host unless the request later opts into a different
policy. One failed target should not abort every other target by default.

## Observability

`deploy` must follow `docs/OBSERVABILITY.md`, but because it is non-HTTP, it
must instrument the real boundaries rather than invent fake request logs.

Required rollout-scoped events:

- action dispatch start and finish
- target resolution summary
- build start and finish
- preflight start and finish per target
- transfer method chosen and outcome per target
- install outcome per target
- restart outcome per target
- verification outcome per target

Sensitive values must never be logged:

- raw params containing credentials
- SSH secrets or agent details
- command arguments that would expose secrets if generic deployment grows later

`deploy` should log enough to explain when a host was unreachable during a
rollout, but it does not own persistent online or offline presence tracking.

## Health

`deploy` is not HTTP-backed, so its health surface should be capability-based.

Minimum useful health signal:

- build tooling is available locally
- SSH inventory can be loaded
- deploy config parses successfully

This is a local capability check, not a remote fleet health system.

## Feature Posture

`deploy` should be feature-gated unless the implementation work deliberately
extends the default synthetic-service exception currently held by `extract`.

Recommended initial posture:

- `lab-apis`: gated feature
- `lab`: pass-through feature, enabled by default with the rest of the product

This keeps it aligned with the standard onboarding model while preserving the
synthetic-service architecture.

## Testing And Verification

`deploy` should satisfy the standard onboarding requirements adapted to its
non-HTTP nature.

### SDK tests

- target resolution
- SSH config parsing
- group expansion
- build command success and failure handling
- transfer fallback selection
- atomic install planning
- restart and verification behavior through mocked boundaries

### Dispatch unit tests

Required standard entry-point tests:

- catalog contains the expected actions
- help returns the catalog
- dispatch round-trips through a mocked deploy client
- unknown action returns `ToolError::UnknownAction`

### MCP adapter tests

- tool delegates to `dispatch::deploy::dispatch`
- destructive metadata remains wired through the shared catalog

### API adapter tests

- route is mounted
- `handle_action` gates `run` without `confirm`
- non-destructive actions succeed without confirmation

### Live verification

When implemented, live evidence should cover at least:

- `targets.list`
- `groups.list`
- `plan`
- `run` against a safe non-critical target
- `verify`

The matching evidence belongs in `docs/coverage/deploy.md`.

## Documentation Updates

The implementation must add or update:

- a new capability doc for the `deploy` service itself
- `docs/coverage/deploy.md`
- `docs/SERVICES.md`
- `docs/CONFIG.md`
- any operator docs that need to distinguish the existing runtime deployment
  topology from the new rollout service

`docs/DEPLOY.md` already documents the device-runtime topology introduced by
`lab serve`. The new `deploy` service must not overload that meaning without
an explicit doc split or clarification.

## Follow-On Capability: `devices`

After `deploy`, the next planned capability is a separate `devices` service or
command focused on device presence and fleet state rather than rollout.

Planned action family:

- `devices.status`
- `devices.events`
- `devices.watch`
- `devices.unreachable`

That capability should own:

- online or offline state
- last-seen tracking
- transition history
- event watch or stream behavior
- optional passive versus active probe policy

That work is intentionally separate from `deploy` so rollout logging does not
turn into a permanent fleet-monitoring subsystem.

## Implementation Sequence

Recommended order after approval:

1. write the `deploy` implementation plan
2. implement the synthetic service and shared dispatch path
3. wire CLI, MCP, API, and metadata
4. verify all-features build and tests
5. update coverage and docs with live evidence where possible
6. begin the `devices` design and implementation plan immediately after the
   `deploy` plan is complete

