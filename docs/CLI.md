# CLI

The CLI is the human-facing surface for `lab`. It should feel thin, predictable, and strongly aligned with the underlying service clients.

## Design Rules

- command parsing belongs in `lab`
- service logic belongs in `lab-apis`
- output formatting belongs in the output layer
- destructive commands require explicit confirmation

## Top-Level Commands

The CLI includes:

- one subcommand per service
- `serve`
- `plugins`
- `install`
- `uninstall`
- `init`
- `health`
- `doctor`
- `completions`
- `self-update`

Representative command tree:

```text
lab
├── <service> ...
├── serve
├── plugins
├── install
├── uninstall
├── init
├── health
├── doctor
├── completions
└── self-update
```

## Per-Service Commands

Each service subcommand should expose operations in a way that mirrors the service model cleanly.

Examples:

- `lab radarr search`
- `lab sonarr series`
- `lab plex libraries`
- `lab unraid array`
- `lab openai models`
- `lab qdrant collections`

The CLI should not invent a second semantic model that drifts from MCP or the SDK.

## Output Formats

Supported output modes are:

- table
- JSON
- compact JSON
- YAML

Rules:

- pretty JSON is acceptable for human-facing TTY output
- compact JSON is preferred for pipes
- tables should be derived from local row wrappers, not SDK types

Rules:

- tables are the default for interactive TTY use
- JSON is the default for pipes and automation
- `lab-apis` types stay presentation-free
- CLI wrappers or local row types handle table rendering

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

Policy knobs may also exist via env, but non-interactive shells should still refuse destructive work unless confirmation has been made explicit.

The CLI reads the same destructive flag from `ActionSpec` that MCP uses for elicitation.

## Multi-Instance Services

The CLI should support explicit instance selection where relevant:

```bash
lab unraid array status --instance shart
```

If there is a clear default instance, that can be used implicitly. Otherwise the command should fail loudly and ask for an instance.

## `lab doctor`

`lab doctor` is a read-only audit command.

It checks:

- env presence
- URL validity
- connectivity
- auth
- service version visibility

It should support:

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

It should expose normalized service health results using the shared `ServiceStatus` model.

## Install and Uninstall

`lab install` and `lab uninstall` handle:

- env validation and prompting
- `.mcp.json` patching
- service enablement changes

These commands are operationally sensitive and should use atomic file writes and backup behavior.

Expected `.mcp.json` behavior:

1. locate the file
2. parse or initialize it
3. compute the updated `--services` list
4. support dry-run diffing
5. back up before mutation
6. write atomically
7. verify the rewritten file parses

## Shell Completions

The CLI should generate completions rather than hand-maintaining shell-specific assets.

## Self-Update

`lab self-update` is explicit only.

Rules:

- no automatic update polling
- update checks happen only when invoked
- replacement should verify release integrity
- GitHub releases are the distribution source
