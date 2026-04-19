# CLI

The CLI is the human-facing surface for `lab`. It must remain thin, predictable, and strongly aligned with the underlying service clients.

## Design Rules

- command parsing belongs in `lab`
- service logic belongs in `lab-apis`
- output formatting belongs in the output layer
- destructive commands require explicit confirmation

## Top-Level Commands

The CLI includes:

- one subcommand per service
- `device`
- `logs`
- `serve`
- `gateway`
- `plugins`
- `install`
- `uninstall`
- `init`
- `health`
- `doctor`
- `audit`
- `scaffold`
- `extract`
- `oauth`
- `help`
- `completions`

Representative command tree:

```text
lab
├── <service> ...
├── device
├── logs
├── serve
├── gateway
├── plugins
├── install
├── uninstall
├── init
├── health
├── doctor
├── audit
├── scaffold
├── extract
├── oauth
├── help
└── completions
```

## Per-Service Commands

Each service subcommand must expose operations in a way that mirrors the service model cleanly.

Examples:

- `lab radarr search`
- `lab sonarr series`
- `lab plex libraries`
- `lab unraid array`
- `lab openai models`
- `lab qdrant collections`

The CLI must not invent a second semantic model that drifts from MCP or the SDK.

## Output Formats

Supported output modes are:

- human-readable terminal output
- JSON

The canonical serialization and output-boundary contract lives in [SERIALIZATION.md](./SERIALIZATION.md).

Rules:

- human-readable output is the default for interactive TTY use
- JSON is the machine-readable mode for pipes and automation
- `lab-apis` types stay presentation-free
- CLI wrappers or local row types handle human rendering

## Color and TTY Behavior

- use `owo-colors`
- disable color automatically when stdout is not a TTY
- honor `NO_COLOR`

## Destructive Operations

Destructive commands use interactive confirmation by default.

Relevant flags:

- `-y` / `--yes`
- `--no-confirm`
- `--dry-run`

Policy knobs may also exist via env, but non-interactive shells must still refuse destructive work unless confirmation has been made explicit.

The CLI reads the same destructive flag from `ActionSpec` that MCP uses for elicitation.

## Multi-Instance Services

The CLI must support explicit instance selection where relevant:

```bash
lab unraid array status --instance shart
```

If there is a clear default instance, that can be used implicitly. Otherwise the command must fail loudly and ask for an instance.

## `lab doctor`

`lab doctor` is a read-only audit command.

It checks:

- env presence
- URL validity
- connectivity
- auth
- service version visibility

It must support:

- all services
- one service
- machine-readable output
- a quicker validation mode

Exit semantics:

- `0` for OK
- `1` for warnings
- `2` for failures

## `lab health`

`lab health` is the product-level health-check surface. It is distinct from repo-level shell helpers.

It must expose normalized service health results using the shared `ServiceStatus` model.

## `lab serve`

`lab serve` is the device runtime entrypoint on every fleet machine.

Rules:

- local hostname plus `[device].master` decide whether the process is `master` or non-master
- the `master` exposes the Web UI, MCP, `/v1/{service}`, `/v1/gateway`, and `/v1/device/*`
- a non-master device exposes only `/health`, `/ready`, and `/v1/device/*`
- non-master startup attempts an initial hello, metadata upload, and bootstrap log flush to the master

## `lab device`

`lab device` is the fleet inventory command group. It routes to the configured master.

Commands:

- `lab device list`
- `lab device get <device_id>`

## `lab logs`

`lab logs` now has two additive paths:

- fleet search routed to the configured master
- local-master log search and bounded follow-up queries against the embedded runtime store

Commands:

- `lab logs search <device_id> <query>`
- `lab logs local search [--subsystem <name>] [--level <level>] [--text <needle>] [--limit <n>]`
- `lab logs local tail [--after-ts <unix_ms>] [--since-event-id <id>] [--limit <n>]`
- `lab logs local stats`
- `lab logs local stream` — exits with guidance to use `GET /v1/logs/stream` or `lab logs local tail`

Rules:

- `lab logs search <device_id> <query>` keeps the existing fleet behavior and continues to use `POST /v1/device/logs/search`
- `lab logs local *` is strictly local-master and uses the shared `dispatch::logs` contract
- true live streaming is not a CLI capability in v1; operators should use `GET /v1/logs/stream` or the gateway-admin `/logs` page
- CLI local log commands stay thin adapters; normalization, retention, search, and tail semantics are owned by `dispatch::logs`

## Install and Uninstall

`lab install` and `lab uninstall` handle:

- env validation and prompting
- `.mcp.json` patching
- service enablement changes

These commands are operationally sensitive and must use atomic file writes and backup behavior.

Expected `.mcp.json` behavior:

1. locate the file
2. parse or initialize it
3. compute the updated `--services` list
4. support dry-run diffing
5. back up before mutation
6. write atomically
7. verify the rewritten file parses

## Shell Completions

The CLI must generate completions rather than hand-maintaining shell-specific assets.

## `lab oauth relay-local`

`lab oauth relay-local` is a browser-side transport helper for OAuth clients that redirect to a
loopback callback but keep the real OAuth listener on another machine.

Supported forms:

```bash
lab oauth relay-local --machine dookie --port 38935
lab oauth relay-local --forward-base http://100.88.16.79:38935/callback/dookie --port 38935
```

Flags:

| Flag | Description |
| --- | --- |
| `--machine <id>` | Resolve the forwarding target from `[oauth.machines.<id>]` in `config.toml`. |
| `--forward-base <url>` | Forward to an explicit callback base URL without a named machine config. |
| `--port <port>` | Loopback port to bind on the browser machine. Required. |
| `--json` | Global flag; emit JSON instead of human-readable tables where applicable. |

Rules:

- exactly one of `--machine` or `--forward-base` is required
- the listener binds only to `127.0.0.1:<port>`
- `--machine` resolves the target from `[oauth.machines.*]` in `config.toml`
- `--forward-base` is for ad hoc use when no named machine exists yet
- the command only forwards the final callback request; it does not mint tokens or run PKCE logic
- the remote callback listener must already be active before the browser callback arrives

Example named-machine config:

```toml
[oauth.machines.dookie]
target_url = "http://100.88.16.79:38935/callback/dookie"
description = "dookie Codex callback listener"
default_port = 38935
```

Runtime behavior:

- incoming callback requests are accepted only on loopback
- the helper forwards the original method, query string, request body, and most headers
- hop-by-hop headers are stripped before forwarding
- successful forwarding returns the upstream response as-is
- failures return transport-oriented HTTP errors on the local loopback callback:
  - unreachable upstream target -> `502`
  - upstream timeout -> `504`
  - unsupported method -> `405`

The device runtime also exposes this relay capability remotely through `POST /v1/device/oauth/relay/start`.
